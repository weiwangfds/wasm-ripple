# Wasm-MQ

[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-2024%20Stable-blue?style=for-the-badge&logo=webassembly)](https://webassembly.org/)

A high-performance, memory-safe message queue library compiled to WebAssembly, designed for modern web applications.

## ✨ Features

- **🚀 High Performance** - Built with Rust and compiled to WebAssembly for near-native speed
- **🧵 Memory Safe** - Rust's ownership model ensures memory safety without garbage collection pauses
- **📡 Topic-based Pub/Sub** - Flexible publish/subscribe pattern with topic-based messaging
- **🔄 Cross-tab Communication** - Seamless message passing between browser tabs via BroadcastChannel API
- **⚡ Synchronous & Async** - Choose between immediate delivery or microtask-based async publishing
- **💾 Ring Buffer Support** - Optional message buffering with O(1) operations and overflow handling
- **📦 Zero-copy Messaging** - Direct JavaScript value passing without serialization overhead
- **🎯 Lightweight** - ~40KB gzipped WebAssembly module

## 📦 Installation

```bash
npm install wasm-mq
```

Or use directly from the `pkg` directory:

```html
<script type="module">
  import init, { MessageQueue } from './pkg/wasm_mq.js';

  await init();
  const mq = new MessageQueue('my-channel');
</script>
```

## 🚀 Quick Start

```javascript
import init, { MessageQueue } from 'wasm-mq';

// Initialize the WASM module
await init();

// Create a message queue with optional channel name for cross-tab communication
const mq = new MessageQueue('my-app');

// Register a topic and get its ID (REQUIRED for all operations)
const topicId = mq.register_topic('events');

// Subscribe to a topic using ID
// Callback receives: payload, topic_id, timestamp, message_id
const subId = mq.subscribe(topicId, (payload, tid, ts, msgId) => {
  console.log('Received:', payload);
});

// Publish a message synchronously using topic ID
mq.publish(topicId, { text: 'Hello, World!', count: 42 });

// Or publish asynchronously (non-blocking)
await mq.publish_async(topicId, { text: 'Async message', data: [1, 2, 3] });

// Batch publish for high throughput
const messages = new Array(100).fill({ data: 'batch' });
mq.publish_batch_by_id(topicId, messages);

// Unsubscribe when done
mq.unsubscribe(topicId, subId);

// Clean up resources
mq.close();
```

## 📊 Performance

Benchmark results on Chrome (Apple M3 Pro):

| Metric | Result | Notes |
| :--- | :--- | :--- |
| **Sync Throughput** | **~5.3M ops/sec** | Zero-allocation hot path |
| **Batch Throughput** | **~7.9M ops/sec** | Optimized batch processing |
| **Latency** | **~0.3 µs** | Ultra-low overhead dispatch |
| **100k Messages** | **~20 ms** | Full processing time |

> **Note**: These results are achieved using the optimized ID-based API and zero-allocation dispatch mechanism.

## 📚 API Reference

### Constructor

```javascript
const mq = new MessageQueue(channelName?: string)
```

- `channelName` (optional): Channel name for cross-tab communication via BroadcastChannel

### Topic Management

```javascript
// Register a topic and get its ID (O(1) lookups)
const topicId = mq.register_topic('my-topic'); // returns number (u32)

// Check if topic exists by ID
const exists = mq.has_topic(topicId); // returns boolean

// Destroy a topic by ID
const destroyed = mq.destroy_topic(topicId); // returns boolean

// Get topic count
const count = mq.topic_count(); // returns number
```

### Subscription

```javascript
// Subscribe to a topic by ID
// Callback signature: (payload, topic_id, timestamp, message_id)
const subId = mq.subscribe(topicId, callback); // returns subscriber ID

// Unsubscribe
const success = mq.unsubscribe(topicId, subId); // returns boolean

// Unsubscribe all
const count = mq.unsubscribe_all(topicId); // returns number of unsubscribed

// Get subscriber count
const count = mq.subscriber_count(topicId); // returns number
```

### Publishing

```javascript
// Synchronous publish (immediate delivery)
mq.publish(topicId, payload);

// Asynchronous publish (delivered in microtask)
await mq.publish_async(topicId, payload);

// Batch publish (highest throughput)
mq.publish_batch_by_id(topicId, [payload1, payload2, ...]);
```

### Ring Buffer Management

```javascript
// Enable message buffering for a topic
mq.enable_topic_buffer(topicId, 100); // capacity (default: 100)

// Check if buffering is enabled
const hasBuffer = mq.has_buffer(topicId); // boolean

// Get buffer size
const size = mq.get_buffer_size(topicId); // current message count

// Get buffer capacity
const capacity = mq.get_buffer_capacity(topicId); // maximum capacity

// Get buffered messages
const messages = mq.get_buffered_messages(topicId); // Array of messages

// Clear buffer
const cleared = mq.clear_buffer(topicId); // number of cleared messages

// Disable buffering
mq.disable_topic_buffer(topicId);
```

### Utilities

```javascript
// Get unique client ID
const clientId = mq.get_client_id(); // string

// Close the queue and release resources
mq.close();
```

## 🔄 Ring Buffer

The ring buffer provides efficient message caching with O(1) operations:

```javascript
const logTopic = mq.register_topic('logs');

// Enable buffer with capacity of 5 messages
mq.enable_topic_buffer(logTopic, 5);

// Publish 10 messages
for (let i = 0; i < 10; i++) {
  mq.publish(logTopic, { id: i, message: `Log ${i}` });
}

// Buffer only keeps the last 5 messages (IDs 5-9)
console.log(mq.get_buffer_size(logTopic)); // 5
console.log(mq.get_buffer_capacity(logTopic)); // 5

// Retrieve buffered messages
const messages = mq.get_buffered_messages(logTopic);
console.log(messages[0].payload.id); // 5 (oldest)
console.log(messages[4].payload.id); // 9 (newest)
```

**Key Features:**
- **Fixed size**: Prevents unbounded memory growth
- **Automatic overflow**: Oldest messages are automatically displaced when full
- **O(1) operations**: Constant time push and retrieval
- **Per-topic**: Each topic can have different buffer settings

## 🌐 Cross-tab Communication

Open the same page in multiple tabs to test cross-tab messaging:

```javascript
// In tab 1
const mq = new MessageQueue('cross-tab-channel');
const topicId = mq.register_topic('updates');
mq.subscribe(topicId, (msg) => console.log('Tab 1 received:', msg));

// In tab 2
const mq = new MessageQueue('cross-tab-channel');
const topicId = mq.register_topic('updates');
mq.publish(topicId, { text: 'Hello from tab 2!' });
// Tab 1 will receive the message!
```

## 🆚 Comparison

| Feature | **wasm-mq** | **Mitt / Tiny-emitter** | **PubSubJS** | **RxJS** |
| :--- | :--- | :--- | :--- | :--- |
| **Sync Throughput** | ~5.3M ops/sec | **~26M ops/sec** | ~18M ops/sec | ~41M ops/sec |
| **Batch Throughput** | ~7.0M ops/sec | **~44M ops/sec** | ~19M ops/sec | ~47M ops/sec |
| **Memory Jitter** | **Low (±0.5 MB)** | Medium (±0.7 MB) | High (±1.0 MB) | High (±0.9 MB) |
| **Cross-tab** | ✅ **Built-in** | ❌ (Manual) | ❌ | ❌ |
| **Buffering** | ✅ **Ring Buffer** | ❌ | ❌ | ✅ (ReplaySubject) |
| **Size (Gzipped)** | ~40KB (WASM) | < 200B | ~3KB | > 20KB |

### When to use which?

1.  **Use `wasm-mq` if:**
    *   You need **Cross-tab Communication** out of the box.
    *   You require **stable memory usage** (low jitter) for long-running apps (dashboards, games).
    *   You need **Message History (Ring Buffer)** for late subscribers.
    *   You are already using Rust/WASM and want zero-overhead communication within WASM.

2.  **Use `mitt` if:**
    *   You just need simple, ultra-fast component communication within a single page.
    *   Bundle size (<200B) is your top priority.

3.  **Use `RxJS` if:**
    *   You need complex functional reactive programming (FRP) operators (map, filter, throttle, debounce).

### 🔬 Running Benchmarks

You can verify these results yourself by running the included benchmark suite:

```bash
# Start local server
npm run serve

# Open in browser
# http://localhost:8000/benchmark/comparison/index.html
```

## 🏗️ Building from Source

```bash
# Install Rust and wasm-pack
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install wasm-pack

# Clone and build
git clone <repo-url>
cd mq-
wasm-pack build --dev --target web

# The compiled files will be in the `pkg/` directory
```

## 📖 Examples

Please refer to the code snippets in the "Quick Start" section above for usage examples.

## 🎯 Use Cases

- **Real-time dashboards** - Broadcast updates across multiple tabs
- **State synchronization** - Keep application state in sync
- **Event logging** - Buffer and replay events with ring buffer
- **Multi-tab coordination** - Coordinate actions between browser tabs
- **Message caching** - Temporarily cache messages for new subscribers

## 🔧 Configuration

### Release Build Optimization

The library is optimized for size in release mode:

```toml
[profile.release]
lto = true           # Link-time optimization
opt-level = "s"      # Optimize for size
strip = true         # Remove debug symbols
codegen-units = 1    # Better optimization at cost of compile time
```

Result: ~40KB gzipped WebAssembly module

## 🧪 Testing

```bash
# Run Rust tests
cargo test
```

## 📝 License

MIT OR Apache-2.0

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📮 Support

For issues and questions, please use the GitHub issue tracker.
