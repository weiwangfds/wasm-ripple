/**
 * Pure TypeScript implementation of Wasm-Ripple functionality
 */

// --- Utilities ---

function generateUUID(): string {
    if (typeof crypto !== 'undefined' && crypto.randomUUID) {
        return crypto.randomUUID();
    }
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
        var r = Math.random() * 16 | 0, v = c == 'x' ? r : (r & 0x3 | 0x8);
        return v.toString(16);
    });
}

// --- Types ---

export interface Message<T = any> {
    id: string;
    topic_id: number;
    origin_id: string;
    payload: T;
    timestamp: number;
}

type SubscriberCallback = (payload: any, topicId: number, timestamp: number, messageId: string) => void;

interface TopicState {
    id: number;
    name: string;
    subscribers: Map<number, SubscriberCallback>;
    buffer: RingBuffer<Message> | null;
    nextSubId: number;
}

// --- Ring Buffer ---

export class RingBuffer<T> {
    private buffer: (T | null)[];
    private capacity: number;
    private size: number;
    private front: number;
    private rear: number;

    constructor(capacity: number) {
        this.capacity = capacity;
        this.buffer = new Array(capacity).fill(null);
        this.size = 0;
        this.front = 0;
        this.rear = 0;
    }

    push(item: T): T | null {
        if (this.capacity === 0) return item;

        let displaced: T | null = null;
        if (this.isFull()) {
            displaced = this.buffer[this.front];
            this.buffer[this.front] = null; // Help GC
            this.front = (this.front + 1) % this.capacity;
        }

        this.buffer[this.rear] = item;
        this.rear = (this.rear + 1) % this.capacity;

        if (displaced === null) {
            this.size++;
        }

        return displaced;
    }

    pop(): T | null {
        if (this.isEmpty()) return null;

        const item = this.buffer[this.front];
        this.buffer[this.front] = null; // Help GC
        this.front = (this.front + 1) % this.capacity;
        this.size--;

        return item;
    }

    peek(): T | null {
        if (this.isEmpty()) return null;
        return this.buffer[this.front];
    }

    isEmpty(): boolean {
        return this.size === 0;
    }

    isFull(): boolean {
        return this.size === this.capacity;
    }

    len(): number {
        return this.size;
    }

    getCapacity(): number {
        return this.capacity;
    }

    clear(): number {
        const count = this.size;
        this.buffer.fill(null);
        this.size = 0;
        this.front = 0;
        this.rear = 0;
        return count;
    }

    getAll(): T[] {
        if (this.isEmpty()) return [];
        
        const result: T[] = [];
        let idx = this.front;
        for (let i = 0; i < this.size; i++) {
            const item = this.buffer[idx];
            if (item !== null) {
                result.push(item);
            }
            idx = (idx + 1) % this.capacity;
        }
        return result;
    }
}

// --- Stream ---

export interface Subscription {
    unsubscribe(): void;
}

export class Stream {
    private _subscribe: (callback: Function) => Function;

    constructor(subscribeFn: (callback: Function) => Function) {
        this._subscribe = subscribeFn;
    }

    subscribe(callback: Function): Subscription {
        const unsubscribeFn = this._subscribe(callback);
        return { unsubscribe: unsubscribeFn };
    }

    map(transform: (value: any) => any): Stream {
        return new Stream(next => {
            return this._subscribe((payload: any, ...args: any[]) => {
                next(transform(payload), ...args);
            });
        });
    }

    filter(predicate: (value: any) => boolean): Stream {
        return new Stream(next => {
            return this._subscribe((payload: any, ...args: any[]) => {
                if (predicate(payload)) next(payload, ...args);
            });
        });
    }

    tap(effect: (value: any) => void): Stream {
        return new Stream(next => {
            return this._subscribe((payload: any, ...args: any[]) => {
                effect(payload);
                next(payload, ...args);
            });
        });
    }

    debounce(ms: number): Stream {
        return new Stream(next => {
            let timeout: any;
            const cleanup = this._subscribe((payload: any, ...args: any[]) => {
                clearTimeout(timeout);
                timeout = setTimeout(() => next(payload, ...args), ms);
            });
            return () => {
                clearTimeout(timeout);
                cleanup();
            };
        });
    }
}

// --- Topic ---

export class Topic {
    private mq: MessageQueue;
    public readonly name: string;
    public readonly id: number;

    constructor(mq: MessageQueue, name: string, id: number) {
        this.mq = mq;
        this.name = name;
        this.id = id;
    }

    stream(): Stream {
        return new Stream(callback => {
            const subId = this.subscribe(callback);
            return () => this.unsubscribe(subId);
        });
    }

    publish(payload: any): void {
        this.mq.publish(this.id, payload);
    }

    async publishAsync(payload: any): Promise<void> {
        return this.mq.publishAsync(this.id, payload);
    }

    publishBatch(payloads: any[]): void {
        this.mq.publishBatchById(this.id, payloads);
    }

    subscribe(callback: SubscriberCallback): number {
        return this.mq.subscribe(this.id, callback);
    }

    unsubscribe(subId: number): boolean {
        return this.mq.unsubscribe(this.id, subId);
    }

    unsubscribeAll(): number {
        return this.mq.unsubscribeAll(this.id);
    }

    get subscriberCount(): number {
        return this.mq.subscriberCount(this.id);
    }

    enableBuffer(capacity?: number): void {
        this.mq.enableTopicBuffer(this.id, capacity);
    }

    disableBuffer(): void {
        this.mq.disableTopicBuffer(this.id);
    }

    clearBuffer(): number {
        return this.mq.clearBuffer(this.id);
    }

    getBufferedMessages(): any[] {
        return this.mq.getBufferedMessages(this.id);
    }

    get bufferSize(): number {
        return this.mq.getBufferSize(this.id);
    }

    get bufferCapacity(): number {
        return this.mq.getBufferCapacity(this.id);
    }

    get hasBuffer(): boolean {
        return this.mq.hasBuffer(this.id);
    }

    destroy(): boolean {
        return this.mq.destroyTopic(this.id);
    }
}

// --- MessageQueue ---

export class MessageQueue {
    private topics: TopicState[] = [];
    private topicIndex: Map<string, number> = new Map();
    private channel: BroadcastChannel | null = null;
    private clientId: string;
    private seenIds: Set<string> = new Set();
    private isClosed: boolean = false;

    constructor(channelName?: string) {
        this.clientId = generateUUID();

        if (channelName && typeof BroadcastChannel !== 'undefined') {
            this.channel = new BroadcastChannel(channelName);
            this.channel.onmessage = this.handleChannelMessage.bind(this);
        }
    }

    // --- Core Logic ---

    registerTopic(name: string): number {
        if (this.topicIndex.has(name)) {
            return this.topicIndex.get(name)!;
        }

        const id = this.topics.length;
        const topicState: TopicState = {
            id,
            name,
            subscribers: new Map(),
            buffer: null,
            nextSubId: 0
        };

        this.topics.push(topicState);
        this.topicIndex.set(name, id);
        return id;
    }

    topic(name: string): Topic {
        const id = this.registerTopic(name);
        return new Topic(this, name, id);
    }

    subscribe(topicId: number, callback: SubscriberCallback): number {
        const topic = this.getTopic(topicId);
        if (!topic) throw new Error(`Topic ID ${topicId} not found`);

        const subId = topic.nextSubId++;
        topic.subscribers.set(subId, callback);
        return subId;
    }

    unsubscribe(topicId: number, subId: number): boolean {
        const topic = this.getTopic(topicId);
        if (!topic) return false;
        return topic.subscribers.delete(subId);
    }

    unsubscribeAll(topicId: number): number {
        const topic = this.getTopic(topicId);
        if (!topic) return 0;
        const count = topic.subscribers.size;
        topic.subscribers.clear();
        return count;
    }

    publish(topicId: number, payload: any): void {
        const msg = this.createMessage(topicId, payload);
        this.dispatchLocal(msg);
        this.broadcast(msg);
    }

    async publishAsync(topicId: number, payload: any): Promise<void> {
        // Use queueMicrotask for async behavior
        return new Promise<void>(resolve => {
            queueMicrotask(() => {
                this.publish(topicId, payload);
                resolve();
            });
        });
    }

    publishBatchById(topicId: number, payloads: any[]): void {
        for (const payload of payloads) {
            this.publish(topicId, payload);
        }
    }

    // --- Buffer Management ---

    enableTopicBuffer(topicId: number, capacity: number = 100): void {
        const topic = this.getTopic(topicId);
        if (topic) {
            topic.buffer = new RingBuffer(capacity);
        }
    }

    disableTopicBuffer(topicId: number): void {
        const topic = this.getTopic(topicId);
        if (topic) {
            topic.buffer = null;
        }
    }

    clearBuffer(topicId: number): number {
        const topic = this.getTopic(topicId);
        return topic && topic.buffer ? topic.buffer.clear() : 0;
    }

    getBufferedMessages(topicId: number): any[] {
        const topic = this.getTopic(topicId);
        if (!topic || !topic.buffer) return [];
        return topic.buffer.getAll().map(msg => msg.payload);
    }

    getBufferSize(topicId: number): number {
        const topic = this.getTopic(topicId);
        return topic && topic.buffer ? topic.buffer.len() : 0;
    }

    getBufferCapacity(topicId: number): number {
        const topic = this.getTopic(topicId);
        return topic && topic.buffer ? topic.buffer.getCapacity() : 0;
    }

    hasBuffer(topicId: number): boolean {
        const topic = this.getTopic(topicId);
        return !!(topic && topic.buffer);
    }

    // --- Internal Helpers ---

    private getTopic(id: number): TopicState | undefined {
        return this.topics[id];
    }

    private createMessage(topicId: number, payload: any): Message {
        return {
            id: generateUUID(),
            topic_id: topicId,
            origin_id: this.clientId,
            payload,
            timestamp: Date.now()
        };
    }

    private dispatchLocal(msg: Message): void {
        if (this.isClosed) return;

        // Store in seen IDs to prevent duplicate processing (though mostly relevant for incoming broadcast)
        this.seenIds.add(msg.id);

        const topic = this.getTopic(msg.topic_id);
        if (topic) {
            // Buffer if enabled
            if (topic.buffer) {
                topic.buffer.push(msg);
            }

            // Notify subscribers
            for (const callback of topic.subscribers.values()) {
                try {
                    callback(msg.payload, msg.topic_id, msg.timestamp, msg.id);
                } catch (e) {
                    console.error('Error in subscriber callback:', e);
                }
            }
        }
    }

    private broadcast(msg: Message): void {
        if (this.channel && !this.isClosed) {
            const topic = this.getTopic(msg.topic_id);
            if (!topic) return;

            // Protocol: [type, payload]
            // type 0: PUB
            // Payload needs to include topic NAME because other tabs might have different IDs
            // So we send a special structure over the wire
            const wireMsg = {
                ...msg,
                topic_name: topic.name // Augment with name for resolution
            };

            this.channel.postMessage([0, wireMsg]);
        }
    }

    private handleChannelMessage(event: MessageEvent): void {
        if (this.isClosed) return;

        const data = event.data;
        if (!Array.isArray(data)) return;

        const [type, payload] = data;

        if (type === 0) { // PUB
            const wireMsg = payload;
            if (this.seenIds.has(wireMsg.id)) return;
            this.seenIds.add(wireMsg.id);

            // Don't process own messages (should be handled by seenIds, but extra check)
            if (wireMsg.origin_id === this.clientId) return;

            // Resolve topic ID locally
            const topicId = this.registerTopic(wireMsg.topic_name);
            
            // Reconstruct local message
            const msg: Message = {
                id: wireMsg.id,
                topic_id: topicId,
                origin_id: wireMsg.origin_id,
                payload: wireMsg.payload,
                timestamp: wireMsg.timestamp
            };

            this.dispatchLocal(msg);
        }
    }

    subscriberCount(topicId: number): number {
        const topic = this.getTopic(topicId);
        return topic ? topic.subscribers.size : 0;
    }

    destroyTopic(topicId: number): boolean {
        // In array-based ID system, we can't easily "remove" without invalidating IDs
        // So we just clear subscribers and buffer, effectively "soft delete"
        const topic = this.getTopic(topicId);
        if (!topic) return false;

        topic.subscribers.clear();
        topic.buffer = null;
        // We keep the name in topicIndex and the slot in topics array
        // to maintain ID consistency for other topics
        return true;
    }

    close(): void {
        this.isClosed = true;
        if (this.channel) {
            this.channel.close();
            this.channel = null;
        }
        this.topics.forEach(t => {
            t.subscribers.clear();
            if (t.buffer) t.buffer.clear();
        });
        this.topics = [];
        this.topicIndex.clear();
        this.seenIds.clear();
    }
}

export default MessageQueue;
