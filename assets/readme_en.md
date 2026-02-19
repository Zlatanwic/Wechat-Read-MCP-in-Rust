# Wechat Article Read MCP in Rust

A Rust-based [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server for reading WeChat Official Account articles.

It renders WeChat article pages via a headless Chrome browser, extracts the title, author, publish date, and body content, and returns structured JSON to AI clients such as Claude Desktop.

## Background

Leading LLM providers like Gemini, Claude, and Minimax all offer web search capabilities, but WeChat's anti-scraping mechanisms prevent them from extracting article content. This project aims to give LLMs the ability to actually read WeChat Official Account articles.

Inspired by [weixin-read-mcp](https://github.com/Bwkyd/wexin-read-mcp), this project is implemented in Rust, with the following advantages:

1. 🚀 **Fast startup** — Compiled to a native binary with no Python interpreter or virtual environment required; MCP service cold-start time drops from seconds to milliseconds
2. 📦 **Easy distribution** — Single executable, no Python / pip / venv installation needed
3. 💾 **Low memory footprint** — No GC overhead; overall memory usage is significantly lower
4. 🔒 **Type safety** — Compile-time type guarantees eliminate runtime type errors
5. ⚡ **Parsing performance** — HTML parsing and text processing are substantially faster in Rust

## Demo

The binary works out of the box:
![Demo](./1.png)

## ✨ Features

- **Single binary** — No Python environment needed; distribute the compiled binary directly
- **MCP protocol** — Integrates with AI clients via stdio JSON-RPC
- **Browser rendering** — Retrieves fully JavaScript-rendered pages using the Chrome DevTools Protocol
- **Structured extraction** — CSS selectors precisely extract article metadata and body content

## 🏗️ Architecture

```
AI Client (Claude Desktop)
    ↕ stdio JSON-RPC (MCP protocol)
main.rs → server.rs → scraper.rs → parser.rs
                          ↕ CDP
                    Chromium (headless)
                          ↓ HTTP
                   mp.weixin.qq.com
```

| Module | Responsibility |
|--------|----------------|
| `main.rs` | Entry point; initializes logging and the MCP service |
| `server.rs` | MCP tool registration, URL validation, response construction |
| `scraper.rs` | Headless Chrome lifecycle management and page navigation |
| `parser.rs` | HTML parsing and CSS selector-based structured data extraction |
| `error.rs` | Unified error type definitions |

## 📋 Prerequisites

1. **Rust toolchain**

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Chrome or Chromium browser**

   ```bash
   # Ubuntu/Debian
   sudo apt install chromium-browser

   # macOS
   brew install --cask chromium

   # Windows
   # Install the Chrome browser
   ```

## 🔨 Build

```bash
# Development build
cargo build

# Release build (optimized, recommended for deployment)
cargo build --release
```

> **Note**: The first compilation of `chromiumoxide` is slow (~60K lines of CDP code); subsequent incremental builds are much faster.

## 🚀 Usage

### Configure Claude Desktop / Cursor / etc

Edit the relevant configuration file:

```json
{
  "mcpServers": {
    "weixin-reader": {
      // Windows
      "command": "/path/to/weixin-mcp-rs.exe"
      // Linux
      // "command": "/path/to/weixin-mcp-rs"
    }
  }
}
```

Replace `/path/to/` with the actual path to the binary.

### Local Testing

```bash
# Run directly (waits for JSON-RPC input on stdin)
cargo run

# Send an MCP initialization request to test
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}' | cargo run 2>/dev/null
```

### MCP Tool Reference

| Tool | Parameters | Description |
|------|------------|-------------|
| `read_weixin_article` | `url: string` | Reads a WeChat article; URL must start with `https://mp.weixin.qq.com/s/` |

**Response format**:

```json
{
  "success": true,
  "title": "Article Title",
  "author": "Author Name",
  "publish_time": "2024-01-01",
  "content": "Article body...",
  "error": null
}
```

## 🔧 Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Chrome not found | Chrome is not installed | Install Chrome, or set the `CHROME` environment variable to point to the executable |
| MCP unresponsive after startup | stdout polluted by log output | Logs are configured to write to stderr; check for stray `print` statements |
| `find_element` timeout | Slow page load or non-WeChat URL | Verify the URL is a valid WeChat article link |
| Empty article content | WeChat anti-scraping triggered | Reduce request frequency |
