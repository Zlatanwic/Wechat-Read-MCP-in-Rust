mod error;
mod parser;
mod scraper;
mod server;

use rmcp::{ServiceExt, transport::stdio};
use server::WeixinServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志（输出到 stderr，避免干扰 stdout 上的 MCP 协议通信）
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting Weixin MCP Server...");

    // 创建服务器实例并通过 stdio 传输启动 MCP 服务
    let service = WeixinServer::new()
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("Failed to start MCP server: {}", e);
        })?;

    // 阻塞等待服务结束
    service.waiting().await?;

    tracing::info!("Weixin MCP Server stopped.");
    Ok(())
}
