# Pure TypeScript Implementation of Wasm-Ripple

This is a pure TypeScript version of the `wasm-ripple` message queue. It provides the same API and features (including cross-tab communication, topic management, and ring buffers) but runs entirely in JavaScript/TypeScript without WebAssembly.

## Usage

```typescript
import { MessageQueue } from './index';

// Initialize
const mq = new MessageQueue('my-app-channel');

// 1. Using Topics (Recommended)
const events = mq.topic('events');

// Subscribe
const sub = events.subscribe((payload, topicId, ts, msgId) => {
    console.log('Received:', payload);
});

// Publish
events.publish({ type: 'login', user: 'alice' });

// Async Publish
await events.publishAsync({ type: 'logout' });

// Stream API
events.stream()
    .filter(msg => msg.type === 'login')
    .map(msg => msg.user)
    .subscribe(user => console.log('User logged in:', user));


// 2. Using Raw IDs (Low-level, High-performance)
const topicId = mq.registerTopic('raw-data');

mq.subscribe(topicId, (data) => {
    // Zero-overhead callback
});

mq.publish(topicId, { value: 42 });
```

## Features

- **Topic-based Pub/Sub**: Efficient ID-based routing.
- **Cross-tab Communication**: Uses `BroadcastChannel` to sync between tabs.
- **Ring Buffer**: O(1) message buffering support.
- **Stream API**: RxJS-like fluent interface for message processing.
- **TypeScript**: Full type safety.
