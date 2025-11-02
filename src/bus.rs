// src/bus.rs

//! # 消息总线模块 (bus)
//!
//! 提供了整个系统的核心通信中枢 `MessageBus`。
//! 这是一个高性能、类型安全的异步发布/订阅实现。

use crate::message::Message;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// ## `AnyChannel` Trait
///
/// 一个内部 trait，用于类型擦除 `tokio::sync::broadcast::Sender<M>`。
/// 这允许我们在一个 `HashMap` 中存储不同消息类型的 `Sender`。
trait AnyChannel: Send + Sync {
    /// 发送一个类型擦除的消息。
    /// 内部会尝试将 `&dyn Any` 向下转型回具体的 `M` 类型。
    fn send_any(&self, msg: &dyn Any) -> Result<usize, Box<dyn Error + Send + Sync>>;
    
    /// 创建一个新的订阅者，返回一个类型擦除的 `Receiver`。
    fn subscribe_any(&self) -> Box<dyn Any + Send>;
}

/// ## `AnyChannel` 实现
///
/// 为泛型的 `broadcast::Sender<M>` 实现 `AnyChannel` trait。
impl<M: Message> AnyChannel for broadcast::Sender<M> {
    fn send_any(&self, msg: &dyn Any) -> Result<usize, Box<dyn Error + Send + Sync>> {
        // 1. 尝试将 `&dyn Any` 向下转型为 `&M`
        let concrete_msg = msg.downcast_ref::<M>().ok_or("Type mismatch")?;
        
        // 2. 发送克隆的消息。如果没有任何订阅者，`send` 会返回 Err，
        //    但在 Pub/Sub 模式中这不应被视为错误，所以我们忽略它。
        Ok(self.send(concrete_msg.clone()).unwrap_or(0))
    }

    fn subscribe_any(&self) -> Box<dyn Any + Send> {
        // 将强类型的 Receiver 包装在 Box<dyn Any> 中返回
        Box::new(self.subscribe())
    }
}

/// ## `MessageBus`
///
/// 系统的中央通信枢纽。
#[derive(Clone)]
pub struct MessageBus {
    /// 核心数据结构：
    /// Key: 消息的 `TypeId`。
    /// Value: 一个类型擦除的 `broadcast::Sender`，包装在 `AnyChannel` trait object 中。
    channels: Arc<RwLock<HashMap<TypeId, Box<dyn AnyChannel>>>>,
    default_capacity: usize,
}

impl MessageBus {
    /// 创建一个新的 `MessageBus` 实例。
    /// `default_capacity`: 为每种新消息类型创建的 broadcast 通道的容量。
    pub fn new(default_capacity: usize) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            default_capacity,
        }
    }

    /// ## `publish`
    ///
    /// 异步发布一个消息到总线。
    ///
    /// - `msg`: 要发布的消息，必须实现 `Message` trait。
    /// - 如果没有订阅者订阅此消息类型，此操作将无声地成功 (返回 `Ok(0)`)。
    /// - 此操作是非阻塞的，发布后立即返回。
    pub async fn publish<M: Message>(&self, msg: M) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let type_id = TypeId::of::<M>();
        let channels = self.channels.read().await; // 获取读锁

        match channels.get(&type_id) {
            Some(channel) => channel.send_any(&msg),
            None => Ok(0), // 没有订阅者，正常返回
        }
    }

    /// ## `subscribe`
    ///
    /// 订阅一种消息类型，返回一个强类型的 `broadcast::Receiver`。
    ///
    /// - `M`: 要订阅的消息类型。
    /// - 如果这是第一次订阅此消息类型，将自动创建一个新的 broadcast 通道。
    /// - 使用了高效的“双重检查锁定”模式来最小化写锁的争用。
    pub async fn subscribe<M: Message>(&self) -> broadcast::Receiver<M> {
        let type_id = TypeId::of::<M>();

        // --- 快速路径：使用读锁 ---
        // 大多数情况下，通道已经存在，此路径将被采用。
        let channels_read = self.channels.read().await;
        if let Some(channel) = channels_read.get(&type_id) {
            return channel
                .subscribe_any()
                .downcast::<broadcast::Receiver<M>>()
                .map(|boxed_rx| *boxed_rx) // 从 Box<Receiver> 中取出 Receiver
                .expect("FATAL: MessageBus internal type corruption. This is a bug.");
        }
        drop(channels_read); // 释放读锁，准备进入慢路径

        // --- 慢路径：使用写锁 ---
        // 仅在通道不存在时才需要获取写锁。
        let mut channels_write = self.channels.write().await;
        
        // **双重检查**：在等待写锁时，可能有另一个线程已经创建了通道。
        if let Some(channel) = channels_write.get(&type_id) {
             return channel
                .subscribe_any()
                .downcast::<broadcast::Receiver<M>>()
                .map(|boxed_rx| *boxed_rx)
                .expect("FATAL: MessageBus internal type corruption. This is a bug.");
        }

        // 通道确实不存在，创建并插入它。
        let (sender, receiver) = broadcast::channel::<M>(self.default_capacity);
        channels_write.insert(type_id, Box::new(sender));
        receiver
    }
}