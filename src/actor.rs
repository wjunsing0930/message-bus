// src/actor.rs

//! # Actor 模块
//!
//! 定义了系统中所有独立组件（Actor）的通用生命周期 trait。

use std::sync::Arc;
use tokio::task::JoinHandle;

/// ## `Actor` Trait
///
/// 为系统中的所有主要组件（如数据引擎、策略、执行引擎）提供统一的接口。
#[async_trait::async_trait]
pub trait Actor: Send + Sync {
    /// 启动 Actor 的主逻辑。
    /// Actor 应该在 `start` 方法内部订阅它所需的消息。
    /// 返回一个 `JoinHandle` 向量，以便主程序可以等待其完成。
    async fn start(self: Arc<Self>) -> Vec<JoinHandle<()>>;
}