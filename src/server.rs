use rmcp::{
    ServerHandler,
    handler::server::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router,
    schemars,
};
use rmcp::handler::server::wrapper::Parameters;
use tracing;

use crate::scraper::WeixinScraper;

/// 工具的请求参数
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadArticleRequest {
    #[schemars(description = "微信文章URL，格式: https://mp.weixin.qq.com/s/xxx")]
    pub url: String,
}

/// MCP 服务器结构体
#[derive(Clone)]
pub struct WeixinServer {
    scraper: std::sync::Arc<WeixinScraper>,
    tool_router: ToolRouter<Self>,
}

impl WeixinServer {
    pub fn new() -> Self {
        Self {
            scraper: std::sync::Arc::new(WeixinScraper::new()),
            tool_router: Self::tool_router(),
        }
    }
}

/// 工具方法定义
#[tool_router]
impl WeixinServer {
    /// 读取微信公众号文章内容
    #[tool(description = "读取微信公众号文章内容。接收一个微信文章 URL，使用浏览器渲染页面并提取结构化内容（标题、作者、发布时间、正文）。URL 必须以 https://mp.weixin.qq.com/s/ 开头。")]
    async fn read_weixin_article(
        &self,
        Parameters(req): Parameters<ReadArticleRequest>,
    ) -> String {
        let url = req.url;

        // 1. URL 校验
        if !url.starts_with("https://mp.weixin.qq.com/s/") {
            let error_msg = format!(
                "Invalid URL format. Must be a Weixin article URL (https://mp.weixin.qq.com/s/xxx). Got: {}",
                url
            );
            tracing::warn!("{}", error_msg);

            return serde_json::json!({
                "success": false,
                "error": error_msg
            }).to_string();
        }

        tracing::info!("Fetching article: {}", url);

        // 2. 调用爬虫获取文章
        match self.scraper.fetch_article(&url).await {
            Ok(article) => {
                tracing::info!("Successfully fetched: {}", article.title);

                serde_json::json!({
                    "success": true,
                    "title": article.title,
                    "author": article.author,
                    "publish_time": article.publish_time,
                    "content": article.content,
                    "error": null
                }).to_string()
            }
            Err(e) => {
                tracing::error!("Failed to fetch article: {}", e);

                serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                }).to_string()
            }
        }
    }
}

/// 实现 MCP 协议处理器
#[tool_handler]
impl ServerHandler for WeixinServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "微信文章阅读器 MCP 服务。提供 read_weixin_article 工具，\
                 可以读取微信公众号文章的标题、作者、发布时间和正文内容。"
                    .into(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}
