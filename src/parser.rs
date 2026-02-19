use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// 解析后的文章数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ArticleData {
    pub title: String,
    pub author: String,
    pub publish_time: String,
    pub content: String,
}

// ── 预编译选择器 ──

fn title_selector() -> &'static Selector {
    static SEL: OnceLock<Selector> = OnceLock::new();
    SEL.get_or_init(|| Selector::parse("h1#activity-name").unwrap())
}

fn author_selector() -> &'static Selector {
    static SEL: OnceLock<Selector> = OnceLock::new();
    SEL.get_or_init(|| Selector::parse("span#js_author_name").unwrap())
}

fn name_selector() -> &'static Selector {
    static SEL: OnceLock<Selector> = OnceLock::new();
    SEL.get_or_init(|| Selector::parse("a#js_name").unwrap())
}

fn publish_time_selector() -> &'static Selector {
    static SEL: OnceLock<Selector> = OnceLock::new();
    SEL.get_or_init(|| Selector::parse("em#publish_time").unwrap())
}

fn content_selector() -> &'static Selector {
    static SEL: OnceLock<Selector> = OnceLock::new();
    SEL.get_or_init(|| Selector::parse("div#js_content").unwrap())
}

// ── 预编译正则 ──

fn newlines_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"\n{3,}").unwrap())
}

fn spaces_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r" {2,}").unwrap())
}

/// 微信文章 HTML 解析器
pub struct WeixinParser;

impl WeixinParser {
    pub fn new() -> Self {
        Self
    }

    /// 解析微信文章 HTML，提取结构化数据
    pub fn parse(&self, html: &str) -> ArticleData {
        let document = Html::parse_document(html);

        let title = self.extract_title(&document);
        let author = self.extract_author(&document);
        let publish_time = self.extract_publish_time(&document);
        let content = self.extract_content(&document);

        ArticleData {
            title,
            author,
            publish_time,
            content,
        }
    }

    /// 提取标题 — h1#activity-name
    fn extract_title(&self, doc: &Html) -> String {
        doc.select(title_selector())
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .unwrap_or_else(|| "未找到标题".to_string())
    }

    /// 提取作者 — 优先 span#js_author_name，回退 a#js_name
    fn extract_author(&self, doc: &Html) -> String {
        if let Some(el) = doc.select(author_selector()).next() {
            let text = el.text().collect::<Vec<_>>().join("").trim().to_string();
            if !text.is_empty() {
                return text;
            }
        }

        doc.select(name_selector())
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .unwrap_or_else(|| "未知作者".to_string())
    }

    /// 提取发布时间 — em#publish_time
    fn extract_publish_time(&self, doc: &Html) -> String {
        doc.select(publish_time_selector())
            .next()
            .map(|el| el.text().collect::<Vec<_>>().join("").trim().to_string())
            .unwrap_or_else(|| "未知时间".to_string())
    }

    /// 提取正文 — div#js_content，移除 script/style，清理空白
    fn extract_content(&self, doc: &Html) -> String {
        match doc.select(content_selector()).next() {
            Some(content_el) => {
                let inner_html = content_el.inner_html();
                let fragment = Html::parse_fragment(&inner_html);

                let text: String = fragment
                    .root_element()
                    .text()
                    .collect::<Vec<_>>()
                    .join("\n");

                self.clean_text(&text)
            }
            None => "未找到正文内容".to_string(),
        }
    }

    /// 清理文本：合并多余换行和空格
    fn clean_text(&self, text: &str) -> String {
        let text = newlines_regex().replace_all(text, "\n\n");
        let text = spaces_regex().replace_all(&text, " ");
        text.trim().to_string()
    }
}
