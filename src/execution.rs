// src/execution.rs

//! # 执行引擎模块 (execution)
//!
//! 模拟与交易所的交互，处理订单请求并产生撮合成交事件。

use crate::actor::Actor;
use crate::bus::MessageBus;
use crate::message::{FillEvent, OrderRequest};
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinHandle;
use tracing::info;

/// ## `SimulatedExecutionEngine`
///
/// - 消费 `OrderRequest` 消息。
/// - 生产 `FillEvent` 消息来模拟成交回报。
pub struct SimulatedExecutionEngine {
    bus: MessageBus,
}

impl SimulatedExecutionEngine {
    pub fn new(bus: MessageBus) -> Self { Self { bus } }
}

#[async_trait::async_trait]
impl Actor for SimulatedExecutionEngine {
    async fn start(self: Arc<Self>) -> Vec<JoinHandle<()>> {
        let mut order_rx = self.bus.subscribe::<OrderRequest>().await;

        let handle = tokio::spawn(async move {
            loop {
                match order_rx.recv().await {
                    Ok(order) => {
                        info!(target: "EXECUTION", "Received {:?}. Simulating fill...", order);
                        // 模拟成交，创建一个 FillEvent
                        let fill = FillEvent {
                            order_id: order.id,
                            symbol: order.symbol.clone(),
                            price: order.price,
                            quantity: order.quantity,
                        };
                        info!(target: "EXECUTION", "Publishing {:?}", fill);
                        if let Err(e) = self.bus.publish(fill).await {
                             tracing::error!(target: "EXECUTION", "Failed to publish fill: {}", e);
                        }
                    },
                    Err(RecvError::Lagged(n)) => tracing::warn!(target: "EXECUTION", "Lagged by {} orders", n),
                    Err(RecvError::Closed) => break,
                }
            }
        });

        vec![handle]
    }
}