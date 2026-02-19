use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use tokio::sync::OnceCell;
use tokio::task::JoinHandle;

use crate::error::AppError;
use crate::parser::{ArticleData, WeixinParser};

/// 浏览器运行状态
struct BrowserState {
    browser: Browser,
    /// handler 必须在后台持续运行，drop 时自动终止
    _handler_task: JoinHandle<()>,
}

/// 浏览器管理器
///
/// 负责管理 headless Chrome 实例的生命周期。
/// 浏览器实例在首次使用时懒初始化，后续复用。
pub struct WeixinScraper {
    parser: WeixinParser,
    browser: OnceCell<BrowserState>,
}

impl WeixinScraper {
    pub fn new() -> Self {
        Self {
            parser: WeixinParser::new(),
            browser: OnceCell::new(),
        }
    }

    /// 确保浏览器已初始化（懒加载模式）
    async fn ensure_browser(&self) -> Result<&Browser, AppError> {
        let state = self
            .browser
            .get_or_try_init(|| async {
                let config = BrowserConfig::builder()
                    .no_sandbox()
                    .window_size(1920, 1080)
                    .arg("--disable-blink-features=AutomationControlled")
                    .build()
                    .map_err(|e| AppError::BrowserError(format!("Config error: {}", e)))?;

                let (browser, mut handler) = Browser::launch(config)
                    .await
                    .map_err(|e| AppError::BrowserError(format!("Launch error: {}", e)))?;

                // handler 必须在后台持续运行，处理 CDP 事件循环
                let handler_task = tokio::spawn(async move {
                    while let Some(_event) = handler.next().await {
                        // 处理浏览器事件
                    }
                });

                Ok::<BrowserState, AppError>(BrowserState {
                    browser,
                    _handler_task: handler_task,
                })
            })
            .await?;

        Ok(&state.browser)
    }

    /// 获取微信文章内容
    pub async fn fetch_article(&self, url: &str) -> Result<ArticleData, AppError> {
        let browser = self.ensure_browser().await?;

        // 创建新页面并导航
        let page = browser
            .new_page(url)
            .await
            .map_err(|e| AppError::BrowserError(format!("New page error: {}", e)))?;

        // 等待关键元素 #js_content 加载
        page.find_element("div#js_content")
            .await
            .map_err(|e| {
                AppError::Timeout(format!(
                    "Waiting for #js_content failed: {}. The page may not be a valid WeChat article.",
                    e
                ))
            })?;

        // 获取渲染后的完整 HTML
        let html = page
            .content()
            .await
            .map_err(|e| AppError::BrowserError(format!("Get content error: {}", e)))?;

        // 关闭页面（释放资源）
        page.close()
            .await
            .map_err(|e| AppError::BrowserError(format!("Close page error: {}", e)))?;

        // 解析 HTML
        let article = self.parser.parse(&html);

        Ok(article)
    }
}
