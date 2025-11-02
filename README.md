用 Rust 构建高性能异步消息总线

行情模块 -> 调用 -> 策略模块.OnNewPrice(...)
策略模块 -> 调用 -> 风控模块.CheckRisk(...)
策略模块 -> 调用 -> 执行模块.PlaceOrder(...)

message-bus/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── actor.rs      定义了系统中所有独立组件（Actor）的通用生命周期 trait
    ├── bus.rs        # 消息总线模块：提供了整个系统的核心通信中枢 `MessageBus`
    ├── data.rs       # 数据引擎模块：模拟一个实时数据源，作为消息的生产者
    ├── execution.rs  # 执行引擎模块：模拟与交易所的交互，处理订单请求并产生撮合成交事件
    ├── message.rs    # 消息模块：定义了系统内部通信所使用的所有消息类型
    └── strategy.rs   # 策略模块：实现交易策略逻辑，是消息的消费者和生产者


<img width="1372" height="768" alt="image" src="https://github.com/user-attachments/assets/8a67da9a-ed81-45b1-8b02-b781bfc54819" />
