# Wasm-Ripple


[![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-2024%20Stable-blue?style=for-the-badge&logo=webassembly)](https://webassembly.org/)

一个高性能、内存安全的消息队列库，编译为 WebAssembly，专为现代 Web 应用程序设计。

## ✨ 特性

- **🚀 高性能** - 使用 Rust 构建，编译为 WebAssembly，接近原生的速度
- **🧵 内存安全** - Rust 的所有权模型确保内存安全，无垃圾回收暂停
- **📡 基于主题的发布/订阅** - 灵活的发布/订阅模式，支持基于主题的消息传递
- **🔄 跨标签页通信** - 通过 BroadcastChannel API 在浏览器标签页之间无缝传递消息
- **⚡ 同步和异步** - 支持立即传递或基于微任务的异步发布
- **💾 环形缓冲区支持** - 可选的消息缓冲，O(1) 操作和溢出处理
- **📦 零拷贝消息传递** - 直接的 JavaScript 值传递，无序列化开销
- **🎯 轻量级** - gzip 压缩后约 40KB 的 WebAssembly 模块

## 📦 安装
 
 ```bash
 npm install @weiwangfds/wasm-ripple
 ```
 
 或直接从 `pkg` 目录使用：
 
 ```html
 <script type="module">
   import init, { MessageQueue } from './pkg/wasm_ripple.js';
 
   await init();
   const mq = new MessageQueue('my-channel');
 </script>
 ```
 
 ## 🚀 快速开始
 
 ```javascript
 import init, { MessageQueue } from '@weiwangfds/wasm-ripple';

// 初始化 WASM 模块
await init();

// 创建消息队列，可选择指定通道名称用于跨标签页通信
const mq = new MessageQueue('my-app');

// 注册主题并获取其 ID（所有操作都必须使用 ID）
const topicId = mq.register_topic('events');

// 使用 ID 订阅主题
// 回调接收参数: payload, topic_id, timestamp, message_id
const subId = mq.subscribe(topicId, (payload, tid, ts, msgId) => {
  console.log('收到消息:', payload);
});

// 使用主题 ID 同步发布消息
mq.publish(topicId, { text: 'Hello, World!', count: 42 });

// 或异步发布（非阻塞）
await mq.publish_async(topicId, { text: '异步消息', data: [1, 2, 3] });

// 批量发布以获得高吞吐量
const messages = new Array(100).fill({ data: 'batch' });
mq.publish_batch_by_id(topicId, messages);

// 完成后取消订阅
mq.unsubscribe(topicId, subId);

// 清理资源
mq.close();
```

## 📊 性能表现

Chrome (Apple M3 Pro) 上的基准测试结果：

| 指标 | 结果 | 说明 |
| :--- | :--- | :--- |
| **同步吞吐量** | **~530万 ops/sec** | 零内存分配热路径 |
| **批量吞吐量** | **~790万 ops/sec** | 优化的批量处理 |
| **延迟** | **~0.3 µs** | 超低开销调度 |
| **10万条消息** | **~20 ms** | 完整处理时间 |

> **注意**：这些结果是使用优化的 ID API 和零内存分配调度机制实现的。

## 📚 API 参考

### 构造函数

```javascript
const mq = new MessageQueue(channelName?: string)
```

- `channelName` (可选): 用于通过 BroadcastChannel 进行跨标签页通信的通道名称

### 主题管理

```javascript
// 注册主题并获取其 ID（O(1) 查找）
const topicId = mq.register_topic('my-topic'); // 返回 number (u32)

// 通过 ID 检查主题是否存在
const exists = mq.has_topic(topicId); // 返回 boolean

// 通过 ID 销毁主题
const destroyed = mq.destroy_topic(topicId); // 返回 boolean

// 获取主题数量
const count = mq.topic_count(); // 返回 number
```

### 订阅

```javascript
// 使用 ID 订阅主题
// 回调签名: (payload, topic_id, timestamp, message_id)
const subId = mq.subscribe(topicId, callback); // 返回订阅者 ID

// 取消订阅
const success = mq.unsubscribe(topicId, subId); // 返回 boolean

// 取消所有订阅
const count = mq.unsubscribe_all(topicId); // 返回取消订阅的数量

// 获取订阅者数量
const count = mq.subscriber_count(topicId); // 返回 number
```

### 发布

```javascript
// 同步发布（立即传递）
mq.publish(topicId, payload);

// 异步发布（在微任务中传递）
await mq.publish_async(topicId, payload);

// 批量发布（最高吞吐量）
mq.publish_batch_by_id(topicId, [payload1, payload2, ...]);
```

### 环形缓冲区管理

```javascript
// 为主题启用消息缓冲
mq.enable_topic_buffer(topicId, 100); // 容量（默认: 100）

// 检查是否启用了缓冲
const hasBuffer = mq.has_buffer(topicId); // boolean

// 获取缓冲区大小
const size = mq.get_buffer_size(topicId); // 当前消息数

// 获取缓冲区容量
const capacity = mq.get_buffer_capacity(topicId); // 最大容量

// 获取缓冲的消息
const messages = mq.get_buffered_messages(topicId); // 消息数组

// 清空缓冲区
const cleared = mq.clear_buffer(topicId); // 清除的消息数量

// 禁用缓冲
mq.disable_topic_buffer(topicId);
```

### 工具方法

```javascript
// 获取唯一的客户端 ID
const clientId = mq.get_client_id(); // string

// 关闭队列并释放资源
mq.close();
```

## 🔄 环形缓冲区

环形缓冲区提供 O(1) 操作的高效消息缓存：

```javascript
const logTopic = mq.register_topic('logs');

// 启用容量为 5 条消息的缓冲区
mq.enable_topic_buffer(logTopic, 5);

// 发布 10 条消息
for (let i = 0; i < 10; i++) {
  mq.publish(logTopic, { id: i, message: `日志 ${i}` });
}

// 缓冲区只保留最后 5 条消息（ID 5-9）
console.log(mq.get_buffer_size(logTopic)); // 5
console.log(mq.get_buffer_capacity(logTopic)); // 5

// 获取缓冲的消息
const messages = mq.get_buffered_messages(logTopic);
console.log(messages[0].payload.id); // 5（最旧的）
console.log(messages[4].payload.id); // 9（最新的）
```

**主要特性：**
- **固定大小**：防止内存无限增长
- **自动溢出**：当缓冲区满时，最旧的消息会被自动替换
- **O(1) 操作**：常量时间的推送和检索
- **每主题独立**：每个主题可以有不同的缓冲设置

## 🌐 跨标签页通信

在多个标签页中打开同一页面来测试跨标签页消息传递：

```javascript
// 在标签页 1
const mq = new MessageQueue('cross-tab-channel');
const topicId = mq.register_topic('updates');
mq.subscribe(topicId, (msg) => console.log('标签页 1 收到:', msg));

// 在标签页 2
const mq = new MessageQueue('cross-tab-channel');
const topicId = mq.register_topic('updates');
mq.publish(topicId, { text: '来自标签页 2 的问候！' });
// 标签页 1 会收到消息！
```

## 🆚 与其他库的对比

| 特性 | **wasm-mq** | **Mitt / Tiny-emitter** | **PubSubJS** | **RxJS** |
| :--- | :--- | :--- | :--- | :--- |
| **同步吞吐量** | ~530万 ops/sec | **~2600万 ops/sec** | ~1800万 ops/sec | ~4100万 ops/sec |
| **批量吞吐量** | ~700万 ops/sec | **~4400万 ops/sec** | ~1900万 ops/sec | ~4700万 ops/sec |
| **内存抖动** | **低 (±0.5 MB)** | 中 (±0.7 MB) | 高 (±1.0 MB) | 高 (±0.9 MB) |
| **跨标签页通信** | ✅ **内置支持** | ❌ (需自行实现) | ❌ | ❌ |
| **消息缓存** | ✅ **内置 Ring Buffer** | ❌ | ❌ | ✅ (ReplaySubject) |
| **大小 (Gzipped)** | ~40KB (WASM) | < 200B | ~3KB | > 20KB |

### 该如何选择？

1.  **选择 `wasm-mq` 如果：**
    *   你需要开箱即用的**跨标签页通信**能力。
    *   你的应用需要**极其稳定的内存表现**（如长期运行的仪表盘、游戏），避免 GC 卡顿。
    *   你需要**消息历史回溯（Ring Buffer）**功能。
    *   你已经在使用 Rust/WASM，并希望在 WASM 内部进行零开销通信。

2.  **选择 `mitt` 如果：**
    *   你只需要在单页面内进行简单、超高速的组件通信。
    *   你对包体积极其敏感（< 200B）。

3.  **选择 `RxJS` 如果：**
    *   你需要复杂的响应式编程操作符（如 map, filter, throttle, debounce）。

### 🔬 运行基准测试

你可以通过运行包含的基准测试套件亲自验证这些结果：

```bash
# 启动本地服务器
npm run serve

# 在浏览器中打开
# http://localhost:8000/benchmark/comparison/index.html
```

## 🏗️ 从源码构建

```bash
# 安装 Rust 和 wasm-pack
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install wasm-pack

# 克隆并构建
git clone <repo-url>
cd mq-
wasm-pack build --dev --target web

# 编译后的文件将在 `pkg/` 目录中
```

## 📖 示例

查看 `demo/` 目录中的工作示例：

- **index.html** - 包含所有功能的交互式演示
- **test.html** - 环形缓冲区测试套件

运行演示：

```bash
# 简单的 HTTP 服务器
python3 -m http.server 8080
# 或
npx serve

# 打开 http://localhost:8080/demo/
```

## 🎯 使用场景

- **实时仪表板** - 在多个标签页之间广播更新
- **状态同步** - 保持应用状态同步
- **事件日志** - 使用环形缓冲区缓冲和重放事件
- **多标签页协调** - 在浏览器标签页之间协调操作
- **消息缓存** - 为新订阅者临时缓存消息

## 🔧 配置

### Release 构建优化

库在 release 模式下针对大小进行了优化：

```toml
[profile.release]
lto = true           # 链接时优化
opt-level = "s"      # 针对大小优化
strip = true         # 移除调试符号
codegen-units = 1    # 以编译时间为代价换取更好的优化
```

结果：gzip 压缩后约 40KB 的 WebAssembly 模块

## 🧪 测试

```bash
# 运行 Rust 测试
cargo test
```

## 📝 许可证

MIT OR Apache-2.0

## 🤝 贡献

欢迎贡献！请随时提交 Pull Request。

## 📮 支持

如有问题和疑问，请使用 GitHub issue tracker。
