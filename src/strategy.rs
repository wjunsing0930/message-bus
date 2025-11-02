// src/strategy.rs

//! # 策略模块 (strategy)
//!
//! 实现交易策略逻辑，是消息的消费者和生产者。

use crate::actor::Actor;
use crate::bus::MessageBus;
use crate::message::{Bar, FillEvent, OrderRequest, OrderSide};
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinHandle;
use tracing::info;
use uuid::Uuid;

/// ## `SimpleTrendFollower`
///
/// 一个简单的趋势跟踪策略 Actor。
/// - 消费 `Bar` 消息来做决策。
/// - 生产 `OrderRequest` 消息来执行交易。
/// - 消费 `FillEvent` 消息来更新内部状态。
pub struct SimpleTrendFollower {
    bus: MessageBus,
    symbol: String,
}

impl SimpleTrendFollower {
    pub fn new(bus: MessageBus, symbol: String) -> Self {
        Self { bus, symbol }
    }

    /// `Bar` 消息的处理逻辑
    async fn handle_bar(&self, bar: Bar) {
        info!(target: "STRATEGY", "Received Bar with close price {}", bar.close);
        if bar.close > 102.0 {
            let order = OrderRequest {
                id: Uuid::new_v4(),
                symbol: self.symbol.clone(),
                side: OrderSide::Buy,
                price: bar.close,
                quantity: 1.0,
            };
            info!(target: "STRATEGY", "Condition met! Publishing {:?}", order);
            if let Err(e) = self.bus.publish(order).await {
                tracing::error!(target: "STRATEGY", "Failed to publish order: {}", e);
            }
        }
    }
    
    /// `FillEvent` 消息的处理逻辑
    async fn handle_fill(&self, fill: FillEvent) {
        info!(target: "STRATEGY", "Received Fill: {:?}. Updating portfolio.", fill);
        // 在真实系统中，这里会更新持仓状态
    }
}

#[async_trait::async_trait]
impl Actor for SimpleTrendFollower {
    async fn start(self: Arc<Self>) -> Vec<JoinHandle<()>> {
        // 订阅 Bar 消息
        let mut bar_rx = self.bus.subscribe::<Bar>().await;
        // 订阅 FillEvent 消息
        let mut fill_rx = self.bus.subscribe::<FillEvent>().await;
        
        let self_clone_for_bar = self.clone();
        let bar_handler = tokio::spawn(async move {
            loop {
                match bar_rx.recv().await {
                    Ok(bar) => {
                        // 过滤掉不关心的 symbol
                        if bar.symbol == self_clone_for_bar.symbol {
                           self_clone_for_bar.handle_bar(bar).await
                        }
                    },
                    Err(RecvError::Lagged(n)) => tracing::warn!(target: "STRATEGY", "Lagged by {} bars", n),
                    Err(RecvError::Closed) => break,
                }
            }
        });
        
        let self_clone_for_fill = self.clone();
        let fill_handler = tokio::spawn(async move {
            loop {
                match fill_rx.recv().await {
                     Ok(fill) => {
                        if fill.symbol == self_clone_for_fill.symbol {
                            self_clone_for_fill.handle_fill(fill).await
                        }
                    },
                    Err(RecvError::Lagged(n)) => tracing::warn!(target: "STRATEGY", "Lagged by {} fills", n),
                    Err(RecvError::Closed) => break,
                }
            }
        });
        
        vec![bar_handler, fill_handler]
    }
}