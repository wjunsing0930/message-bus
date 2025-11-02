// src/message.rs

//! # 消息模块 (message)
//!
//! 定义了系统内部通信所使用的所有消息类型。
//! 它们是整个事件驱动架构的血液。

use std::fmt::Debug;
use uuid::Uuid;

/// ## `Message` Trait
///
/// 所有消息类型都必须实现的标记 trait。
/// `Clone`: 允许消息在 broadcast 通道中被克隆给多个订阅者。
/// `Debug`: 便于日志记录和调试。
/// `Send + Sync + 'static`: 确保消息可以在多线程/多任务环境中安全地传递。
pub trait Message: Clone + Debug + Send + Sync + 'static {}

// --- 行情数据消息 ---

#[derive(Clone, Debug)]
pub struct Bar {
    pub id: Uuid,
    pub ts_event: u64,
    pub symbol: String,
    pub close: f64,
}
impl Message for Bar {}

// --- 交易执行消息 ---

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Debug)]
pub struct OrderRequest {
    pub id: Uuid,
    pub symbol: String,
    pub side: OrderSide,
    pub price: f64,
    pub quantity: f64,
}
impl Message for OrderRequest {}

#[derive(Clone, Debug)]
pub struct FillEvent {
    pub order_id: Uuid,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
}
impl Message for FillEvent {}