# 🚀 Rust + WebAssembly 打造高性能前端消息队列：Wasm-Ripple

在现代前端开发中，随着应用复杂度的提升，组件通信、跨标签页数据同步以及高性能数据流处理变得愈发重要。虽然 JavaScript 提供了 `EventTarget`、`BroadcastChannel` 等原生方案，但在处理高频消息、保证内存安全以及追求极致性能方面，纯 JS 方案往往面临瓶颈。

今天，我要向大家介绍一个基于 **Rust** 编写并编译为 **WebAssembly** 的高性能消息队列库 —— **Wasm-Ripple**。它不仅带来了接近原生的运行速度，还利用 Rust 的所有权机制确保了内存安全，是构建高性能 Web 应用的理想选择。

> **项目地址**：[https://github.com/weiwangfds/wasm-ripple](https://github.com/weiwangfds/wasm-ripple)
> **NPM 安装**：`npm install @weiwangfds/wasm-ripple`

---

## 🌟 为什么选择 Wasm-Ripple？

在 Web 开发中，我们经常遇到以下痛点：
1.  **性能瓶颈**：在高频数据更新（如股市行情、实时游戏、日志监控）场景下，JS 的事件循环和垃圾回收（GC）可能导致卡顿。
2.  **跨标签页通信繁琐**：虽然有 `BroadcastChannel`，但在不同组件间优雅地管理订阅和状态并不容易。
3.  **内存泄漏风险**：复杂的事件监听如果忘记解绑，很容易导致内存泄漏。

**Wasm-Ripple** 正是为了解决这些问题而生：

*   **⚡ 极致性能**：核心逻辑由 Rust 编写，编译为 WASM，同步吞吐量可达 **530万 ops/sec**，批量处理更是高达 **790万 ops/sec**。
*   **🛡️ 内存安全**：得益于 Rust 的所有权模型，无 GC 暂停，内存管理更加可控且安全。
*   **📦 零拷贝开销**：精心设计的数据传递机制，直接传递 JavaScript 值，避免了昂贵的序列化/反序列化开销。
*   **🔄 跨标签页同步**：内置基于 `BroadcastChannel` 的通信机制，多窗口数据同步开箱即用。
*   **💾 环形缓冲区**：提供 O(1) 复杂度的消息缓存，支持自动覆盖旧数据，非常适合日志记录和实时图表。

---

## 📊 性能实测

在 Chrome (Apple M3 Pro) 上的基准测试结果令人印象深刻：

| 指标 | 结果 | 说明 |
| :--- | :--- | :--- |
| **同步吞吐量** | **~530万 ops/sec** | 零内存分配热路径，极致速度 |
| **批量吞吐量** | **~790万 ops/sec** | 优化的批量处理能力 |
| **延迟** | **~0.3 µs** | 超低开销的消息调度 |
| **10万条消息** | **~20 ms** | 完整处理时间，几乎无感 |

> *注：这些数据基于优化的 ID API 和零内存分配调度机制。*

---

## 📚 常见方案选型参考

在选择消息队列库时，我们需要根据实际业务场景进行权衡。以下是 Wasm-Ripple 与其他主流方案的横向对比：

| 特性 | **Wasm-Ripple** | **Mitt / Tiny-Emitter** | **PubSubJS** | **RxJS** |
| :--- | :--- | :--- | :--- | :--- |
| **同步吞吐量** | ~530万 ops/sec | **~2600万 ops/sec** | ~1800万 ops/sec | ~4100万 ops/sec |
| **批量吞吐量** | ~790万 ops/sec | **~4400万 ops/sec** | ~1900万 ops/sec | ~4700万 ops/sec |
| **内存抖动** | **低 (±0.5 MB)** | 中 (±0.7 MB) | 高 (±1.0 MB) | 高 (±0.9 MB) |
| **跨标签页通信** | **✅ 内置支持** | ❌ 需额外实现 | ❌ | ❌ |
| **数据缓冲** | **✅ 环形缓冲区 (O(1))** | ❌ 无 | ❌ 无 | ✅ 支持 (ReplaySubject) |
| **体积 (Gzip)** | ~40KB (WASM) | **< 200B** | ~3KB | > 20KB |
| **适用场景** | **稳定低抖动、跨窗口** | 极致轻量、单机高频 | 通用发布订阅 | 复杂流处理 |

**选型建议：**

*   如果你需要一个**极致轻量**（几百字节）且追求**单机极致吞吐量**的库，**Mitt** 是首选。
*   如果你需要处理**复杂的异步流、操作符转换**，**RxJS** 是无可替代的王者。
*   但如果你关注**低内存抖动（GC 压力小）**，或者需要开箱即用的**跨标签页通信**和**高效消息缓冲**（如实时日志、高频数据流），那么 **Wasm-Ripple** 将是你的最佳选择。

---

## ⚖️ 优缺点分析

为了让大家更客观地评估 Wasm-Ripple 是否适合自己的项目，我们总结了它的优缺点：

### ✅ 优点 (Pros)

1.  **极致的性能表现**：利用 Rust 编译为 WASM，避开了 JS 引擎的某些开销，特别是在高频消息处理上表现优异（5M+ ops/sec）。
2.  **确定性的内存管理**：无 GC 暂停（Stop-the-World），对于对延迟敏感的应用（如游戏、高频交易界面）至关重要。
3.  **开箱即用的跨标签页通信**：无需引入额外的库或编写复杂的 `BroadcastChannel` 封装代码，一行配置即可开启。
4.  **高效的数据缓冲**：内置环形缓冲区（Ring Buffer），在处理日志、实时图表等需要“滑动窗口”数据的场景下，性能远超 JS 数组操作。
5.  **零拷贝优化**：在 JS 和 WASM 之间传递对象时，采用了引用传递而非序列化，大大降低了 CPU 消耗。

### ❌ 缺点 (Cons)

1.  **引入成本**：需要加载 WASM 模块（虽然只有 ~40KB），对于极致追求首屏加载速度且无复杂通信需求的简单页面，可能略显“重”了。
2.  **异步加载**：必须等待 `init()` 完成后才能使用，这与纯 JS 库的同步导入使用习惯略有不同。
3.  **生态系统**：相比于 Redux、RxJS 等老牌库，周边生态和插件尚在发展中。

---

## 💡 深度适用场景

Wasm-Ripple 并非为了取代所有事件库，但在以下场景中，它能发挥不可替代的作用：

### 1. 金融与加密货币交易终端
**场景**：需要实时接收并渲染成百上千个交易对的 Order Book（订单簿）和 Trade History（成交历史）。
**优势**：高吞吐量确保数据不积压，无 GC 暂停确保界面不卡顿，环形缓冲区可直接用于 K 线图数据的缓存。

### 2. 复杂的前端日志与监控系统
**场景**：在用户浏览器端收集大量埋点、错误日志和性能数据，并进行预处理或批量上报。
**优势**：利用环形缓冲区暂存日志，避免突发流量撑爆内存；利用 WASM 的高效处理能力在前端进行初步的数据清洗。

### 3. Web 游戏与即时战略 (RTS) 游戏
**场景**：大量的游戏实体（单位、子弹、特效）之间的状态同步和事件通知。
**优势**：极低的延迟和稳定的帧率表现，确保游戏逻辑流畅运行。

### 4. 多窗口协作工具 (Figma-like)
**场景**：用户打开多个标签页编辑同一个文档，需要实时同步鼠标位置、选中状态和操作指令。
**优势**：内置的跨标签页通信机制让多窗口同步变得简单且高效。

---

## 🛠️ 快速上手

### 1. 安装

通过 npm 安装：

```bash
npm install @weiwangfds/wasm-ripple
```

### 2. 初始化与基本使用

Wasm-Ripple 的 API 设计非常直观，支持 **Topic（主题）** 模式。

```javascript
import init, { MessageQueue } from '@weiwangfds/wasm-ripple';

async function main() {
    // 1. 初始化 WASM 模块
    await init();

    // 2. 创建消息队列实例
    // 可选参数 'my-app' 用于开启跨标签页通信
    const mq = new MessageQueue('my-app');

    // 3. 注册主题（必须步骤）
    // 注册后会返回一个唯一的 Topic ID，后续操作都基于这个 ID，性能最高
    const topicId = mq.register_topic('user-events');

    // 4. 订阅主题
    const subId = mq.subscribe(topicId, (payload, tid, ts, msgId) => {
        console.log(`收到消息 [${ts}]:`, payload);
    });

    // 5. 发布消息
    // 支持传递任意 JS 对象
    mq.publish(topicId, { type: 'login', userId: 1001 });

    // 6. 异步发布（非阻塞）
    await mq.publish_async(topicId, { type: 'action', data: 'click' });
    
    // 7. 批量发布（高性能场景推荐）
    const batchData = Array(100).fill({ data: 'test' });
    mq.publish_batch_by_id(topicId, batchData);

    // 8. 清理资源
    mq.unsubscribe(topicId, subId);
    mq.close();
}

main();
```

### 3. 高级特性：环形缓冲区 (Ring Buffer)

如果你需要缓存最近的 N 条消息（例如实时日志窗口），Wasm-Ripple 的环形缓冲区非常有用：

```javascript
// 启用缓冲区，容量为 5
mq.enable_topic_buffer(topicId, 5);

// 发布 10 条消息
for (let i = 0; i < 10; i++) {
    mq.publish(topicId, `Log ${i}`);
}

// 获取缓冲区中的消息
const logs = mq.get_buffered_messages(topicId);
console.log(logs.length); // 5
console.log(logs[0].payload); // "Log 5" (自动丢弃了旧消息)
```

---

## 🔗 链接与资源

Wasm-Ripple 致力于为 Web 提供高性能的基础设施。如果你对性能有极致追求，或者想尝试 Rust + WebAssembly 的开发体验，欢迎试用并 Star！

*   **GitHub 仓库**: [https://github.com/weiwangfds/wasm-ripple](https://github.com/weiwangfds/wasm-ripple)
*   **NPM 地址**: [https://www.npmjs.com/package/@weiwangfds/wasm-ripple](https://www.npmjs.com/package/@weiwangfds/wasm-ripple)

欢迎大家在评论区交流使用心得！👇
