// src/main.rs

//! # 主程序 (main)
//!
//! 负责组装和启动整个系统，是所有组件的编排器。

// 声明所有模块
mod actor;
mod bus;
mod data;
mod execution;
mod message;
mod strategy;

use actor::Actor;
use bus::MessageBus;
use data::SimulatedDataEngine;
use execution::SimulatedExecutionEngine;
use strategy::SimpleTrendFollower;

use futures::future::join_all;
use std::sync::Arc;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() {
    // --- 1. 初始化 ---
    // 设置日志，可以通过 RUST_LOG 环境变量控制级别, e.g., RUST_LOG=info,data=debug
    fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .with_target(true) // 打印 target
        .init();

    // 创建核心 MessageBus
    let bus = MessageBus::new(1024);
    let symbol = "BTC-USD".to_string();

    // --- 2. 组装 Actors ---
    // 将所有 Actor 放入一个向量中，便于统一管理
    let actors: Vec<Arc<dyn Actor>> = vec![
        Arc::new(SimulatedDataEngine::new(bus.clone(), symbol.clone())),
        Arc::new(SimpleTrendFollower::new(bus.clone(), symbol.clone())),
        Arc::new(SimulatedExecutionEngine::new(bus.clone())),
    ];

    info!(target: "MAIN", "System starting up...");

    // --- 3. 启动 Actors ---
    // 启动所有 Actor 并收集它们的任务句柄
    let mut handles = Vec::new();
    for actor in actors {
        handles.extend(actor.start().await);
    }

    info!(target: "MAIN", "All actors started. Running for 5 seconds...");
    tokio::time::sleep(Duration::from_secs(5)).await;

    // --- 4. 优雅关闭 ---
    info!(target: "MAIN", "Shutting down...");
    for handle in &handles {
        handle.abort(); // 中止所有后台任务
    }
    // 等待所有任务确认中止
    let _ = join_all(handles).await;
    
    info!(target: "MAIN", "System shut down gracefully.");
}