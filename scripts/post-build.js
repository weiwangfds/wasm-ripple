const fs = require('fs');
const path = require('path');

const pkgDir = path.join(__dirname, '../pkg');
const jsFile = path.join(pkgDir, 'wasm_ripple.js');
const dtsFile = path.join(pkgDir, 'wasm_ripple.d.ts');

const streamJs = `
/**
 * A lightweight Stream implementation for fluent message processing
 */
export class Stream {
    constructor(subscribeFn) {
        this._subscribe = subscribeFn;
    }

    /**
     * Subscribe to the stream
     * @param {Function} callback
     * @returns {{ unsubscribe: Function }} Subscription object
     */
    subscribe(callback) {
        const unsubscribeFn = this._subscribe(callback);
        return { unsubscribe: unsubscribeFn };
    }

    /**
     * Transform values in the stream
     * @param {Function} transform - (value) => newValue
     * @returns {Stream}
     */
    map(transform) {
        return new Stream(next => {
            return this._subscribe((payload, ...args) => {
                next(transform(payload), ...args);
            });
        });
    }

    /**
     * Filter values in the stream
     * @param {Function} predicate - (value) => boolean
     * @returns {Stream}
     */
    filter(predicate) {
        return new Stream(next => {
            return this._subscribe((payload, ...args) => {
                if (predicate(payload)) next(payload, ...args);
            });
        });
    }

    /**
     * Perform side-effect without changing the value
     * @param {Function} effect - (value) => void
     * @returns {Stream}
     */
    tap(effect) {
        return new Stream(next => {
            return this._subscribe((payload, ...args) => {
                effect(payload);
                next(payload, ...args);
            });
        });
    }

    /**
     * Debounce values by a specified time
     * @param {number} ms - milliseconds
     * @returns {Stream}
     */
    debounce(ms) {
        return new Stream(next => {
            let timeout;
            const cleanup = this._subscribe((payload, ...args) => {
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
`;

const topicJs = `
/**
 * A wrapper class for topic operations
 */
export class Topic {
    constructor(mq, name, id) {
        this.mq = mq;
        this.name = name;
        this.id = id;
    }

    /**
     * Create a Stream from this topic for fluent processing
     * @returns {Stream}
     */
    stream() {
        return new Stream(callback => {
            const subId = this.subscribe(callback);
            return () => this.unsubscribe(subId);
        });
    }

    /**
     * Publish a message to this topic
     * @param {any} payload
     */
    publish(payload) {
        this.mq.publish(this.id, payload);
    }

    /**
     * Publish a message asynchronously
     * @param {any} payload
     * @returns {Promise<any>}
     */
    async publishAsync(payload) {
        return this.mq.publish_async(this.id, payload);
    }

    /**
     * Publish multiple messages efficiently
     * @param {Array<any>} payloads
     */
    publishBatch(payloads) {
        this.mq.publish_batch_by_id(this.id, payloads);
    }

    /**
     * Subscribe to this topic
     * @param {Function} callback - (payload, topic_id, timestamp, message_id) => void
     * @returns {number} subscription ID
     */
    subscribe(callback) {
        return this.mq.subscribe(this.id, callback);
    }

    /**
     * Unsubscribe from this topic
     * @param {number} sub_id
     * @returns {boolean}
     */
    unsubscribe(sub_id) {
        return this.mq.unsubscribe(this.id, sub_id);
    }

    /**
     * Unsubscribe all subscribers from this topic
     * @returns {number} number of subscribers removed
     */
    unsubscribeAll() {
        return this.mq.unsubscribe_all(this.id);
    }

    /**
     * Get the number of subscribers
     * @returns {number}
     */
    get subscriberCount() {
        return this.mq.subscriber_count(this.id);
    }

    /**
     * Enable buffering for this topic
     * @param {number} [capacity]
     */
    enableBuffer(capacity) {
        this.mq.enable_topic_buffer(this.id, capacity);
    }

    /**
     * Disable buffering for this topic
     */
    disableBuffer() {
        this.mq.disable_topic_buffer(this.id);
    }

    /**
     * Clear the buffer
     * @returns {number} number of messages cleared
     */
    clearBuffer() {
        return this.mq.clear_buffer(this.id);
    }

    /**
     * Get buffered messages
     * @returns {Array<any>}
     */
    getBufferedMessages() {
        return this.mq.get_buffered_messages(this.id);
    }

    /**
     * Get buffer size
     * @returns {number}
     */
    get bufferSize() {
        return this.mq.get_buffer_size(this.id);
    }

    /**
     * Get buffer capacity
     * @returns {number}
     */
    get bufferCapacity() {
        return this.mq.get_buffer_capacity(this.id);
    }
    
    /**
     * Check if buffering is enabled
     * @returns {boolean}
     */
    get hasBuffer() {
        return this.mq.has_buffer(this.id);
    }

    /**
     * Destroy this topic
     * @returns {boolean}
     */
    destroy() {
        return this.mq.destroy_topic(this.id);
    }
}

/**
 * Get or create a topic wrapper
 * @param {string} name
 * @returns {Topic}
 */
MessageQueue.prototype.topic = function(name) {
    const id = this.register_topic(name);
    return new Topic(this, name, id);
};
`;

const streamDts = `
export interface Subscription {
  unsubscribe(): void;
}

export class Stream {
  constructor(subscribeFn: (callback: Function) => Function);
  
  subscribe(callback: Function): Subscription;
  map(transform: (value: any) => any): Stream;
  filter(predicate: (value: any) => boolean): Stream;
  tap(effect: (value: any) => void): Stream;
  debounce(ms: number): Stream;
}
`;

const topicDts = `
export class Topic {
  constructor(mq: MessageQueue, name: string, id: number);
  readonly id: number;
  readonly name: string;
  
  stream(): Stream;
  publish(payload: any): void;
  publishAsync(payload: any): Promise<any>;
  publishBatch(payloads: Array<any>): void;
  subscribe(callback: Function): number;
  unsubscribe(sub_id: number): boolean;
  unsubscribeAll(): number;
  
  readonly subscriberCount: number;
  readonly bufferSize: number;
  readonly bufferCapacity: number;
  readonly hasBuffer: boolean;
  
  enableBuffer(capacity?: number): void;
  disableBuffer(): void;
  clearBuffer(): number;
  getBufferedMessages(): Array<any>;
  destroy(): boolean;
}
`;

const topicMethodDts = `
  /**
   * Get or create a topic wrapper
   */
  topic(name: string): Topic;
`;

// Process JS file
if (fs.existsSync(jsFile)) {
    let content = fs.readFileSync(jsFile, 'utf8');
    if (!content.includes('export class Topic')) {
        // Insert before the exports
        const exportIdx = content.lastIndexOf('export { initSync };');
        if (exportIdx !== -1) {
            content = content.slice(0, exportIdx) + streamJs + '\n' + topicJs + '\n' + content.slice(exportIdx);
            fs.writeFileSync(jsFile, content);
            console.log('Updated wasm_ripple.js with Stream and Topic class');
        } else {
            console.error('Could not find export statement in wasm_ripple.js');
        }
    } else {
        console.log('Topic class already exists in wasm_ripple.js');
    }
} else {
    console.error('wasm_ripple.js not found');
}

// Process DTS file
if (fs.existsSync(dtsFile)) {
    let content = fs.readFileSync(dtsFile, 'utf8');
    
    if (!content.includes('export class Topic')) {
        // Add Topic class definition before MessageQueue
        const mqIdx = content.indexOf('export class MessageQueue');
        if (mqIdx !== -1) {
            content = content.slice(0, mqIdx) + streamDts + '\n' + topicDts + '\n' + content.slice(mqIdx);
            
            // Add topic method to MessageQueue
            const lastBraceIdx = content.lastIndexOf('}');
            if (lastBraceIdx !== -1) {
                // Find the closing brace of MessageQueue class. 
                // Since MessageQueue is likely the last class or we search for the specific closing brace?
                // A safer way is to replace the end of MessageQueue definition
                // But simplified: search for last '}' in file might be risky if there are other exports after
                // Let's find "subscribe(topic_id: number, callback: Function): number;" and insert after it
                
                const subMethod = 'subscribe(topic_id: number, callback: Function): number;';
                const subIdx = content.indexOf(subMethod);
                if (subIdx !== -1) {
                    const insertPos = subIdx + subMethod.length;
                    content = content.slice(0, insertPos) + '\n' + topicMethodDts + content.slice(insertPos);
                    fs.writeFileSync(dtsFile, content);
                    console.log('Updated wasm_ripple.d.ts with Topic definitions');
                } else {
                    console.error('Could not find subscribe method in wasm_ripple.d.ts');
                }
            }
        } else {
            console.error('Could not find MessageQueue class in wasm_ripple.d.ts');
        }
    } else {
        console.log('Topic class already exists in wasm_ripple.d.ts');
    }
} else {
    console.error('wasm_ripple.d.ts not found');
}
