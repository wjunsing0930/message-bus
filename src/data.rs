// src/data.rs

//! # 数据引擎模块 (data)
//!
//! 模拟一个实时数据源，作为消息的生产者。

use crate::actor::Actor;
use crate::bus::MessageBus;
use crate::message::Bar;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing::info;
use uuid::Uuid;

/// ## `SimulatedDataEngine`
///
/// 一个 Actor，周期性地生成 `Bar` 消息并将其发布到 `MessageBus`。
pub struct SimulatedDataEngine {
    bus: MessageBus,
    symbol: String,
}

impl SimulatedDataEngine {
    pub fn new(bus: MessageBus, symbol: String) -> Self {
        Self { bus, symbol }
    }
}

#[async_trait::async_trait]
impl Actor for SimulatedDataEngine {
    async fn start(self: Arc<Self>) -> Vec<JoinHandle<()>> {
        let handle = tokio::spawn(async move {
            let mut price = 100.0;
            loop {
                let bar = Bar {
                    id: Uuid::new_v4(),
                    ts_event: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos() as u64,
                    symbol: self.symbol.clone(),
                    close: price,
                };

                info!(target: "DATA", "Publishing {:?}", bar);
                if let Err(e) = self.bus.publish(bar).await {
                    tracing::error!(target: "DATA", "Failed to publish bar: {}", e);
                }
                
                price += 1.0;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        });
        vec![handle]
    }
}