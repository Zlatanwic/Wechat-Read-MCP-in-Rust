use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    /// URL 格式不正确
    #[error("Invalid URL format: {0}. Must be https://mp.weixin.qq.com/s/xxx")]
    InvalidUrl(String),

    /// 浏览器操作失败
    #[error("Browser error: {0}")]
    BrowserError(String),

    /// 页面加载超时
    #[error("Page load timeout: {0}")]
    Timeout(String),

    /// HTML 解析失败
    #[error("Parse error: {0}")]
    ParseError(String),

    /// chromiumoxide 错误
    #[error("Chromium error: {0}")]
    ChromiumError(#[from] chromiumoxide::error::CdpError),
}
