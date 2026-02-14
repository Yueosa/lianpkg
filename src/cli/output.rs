//! 格式化输出模块
//!
//! 提供美化的终端输出，支持表格、颜色、Box 等

use super::logger;
use std::path::Path;
use std::sync::Mutex;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// ============================================================================
// 进度条状态管理
// ============================================================================

/// 当前进度条状态
#[derive(Default)]
struct ProgressState {
    active: bool,
    label: String,
    current: usize,
    total: usize,
}

static PROGRESS_STATE: Mutex<ProgressState> = Mutex::new(ProgressState {
    active: false,
    label: String::new(),
    current: 0,
    total: 0,
});

// ============================================================================
// 字符串工具
// ============================================================================

/// 计算单个字符的显示宽度
fn char_width(c: char) -> usize {
    // unicode-width 的 .width() 会返回 Option<usize>，不可打印字符返回 None
    c.width().unwrap_or(0)
}

/// 去除字符串中的 ANSI 转义序列
fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // 跳过 ANSI 转义序列: ESC [ ... m
            if chars.peek() == Some(&'[') {
                chars.next(); // 消费 '['
                              // 跳过直到遇到 'm'
                while let Some(&ch) = chars.peek() {
                    chars.next();
                    if ch == 'm' {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// 计算字符串的显示宽度（自动过滤 ANSI 转义序列）
fn display_width(s: &str) -> usize {
    // 先去除 ANSI 序列，再调用 unicode-width 的字符串扩展方法
    let stripped = strip_ansi(s);
    UnicodeWidthStr::width(stripped.as_str())
}

/// 按显示宽度截断字符串（UTF-8 安全）
fn truncate_str(s: &str, max_width: usize) -> String {
    let current_width = display_width(s);

    // 不需要截断
    if current_width <= max_width {
        return s.to_string();
    }

    // 需要截断，保留 "..." (3个字符宽度)
    if max_width < 4 {
        return ".".repeat(max_width);
    }

    let mut width = 0;
    let mut result = String::new();
    let target_width = max_width - 3; // 为 "..." 保留空间

    for c in s.chars() {
        let cw = char_width(c);
        if width + cw > target_width {
            break;
        }
        width += cw;
        result.push(c);
    }

    result.push_str("...");
    result
}

/// 按显示宽度填充字符串（右侧补空格）

// ============================================================================
// 颜色与样式
// ============================================================================

/// ANSI 颜色代码
pub mod color {
    pub const RESET: &str = "\x1b[0m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";

    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    #[allow(dead_code)]
    pub const WHITE: &str = "\x1b[37m";

    #[allow(dead_code)]
    pub const BG_RED: &str = "\x1b[41m";
    #[allow(dead_code)]
    pub const BG_GREEN: &str = "\x1b[42m";
    #[allow(dead_code)]
    pub const BG_BLUE: &str = "\x1b[44m";
}

/// 检查是否支持颜色输出
pub fn supports_color() -> bool {
    // 简单检测：如果是 tty 则支持
    use std::io::IsTerminal;
    std::io::stdout().is_terminal()
}

/// 条件性添加颜色
fn colorize(text: &str, code: &str) -> String {
    if supports_color() {
        format!("{}{}{}", code, text, color::RESET)
    } else {
        text.to_string()
    }
}

// ============================================================================
// 基础输出函数
// ============================================================================

/// 检查是否为 quiet 模式
pub fn is_quiet() -> bool {
    logger::is_quiet()
}

/// 输出标题 (quiet 模式下不输出)
pub fn title(text: &str) {
    if is_quiet() {
        return;
    }
    let text_width = display_width(text);
    let line = "═".repeat(text_width + 4);
    println!();
    println!("{}", colorize(&line, color::CYAN));
    println!(
        "{}",
        colorize(
            &format!("  {}  ", text),
            &format!("{}{}", color::BOLD, color::CYAN)
        )
    );
    println!("{}", colorize(&line, color::CYAN));
}

/// 输出子标题 (quiet 模式下不输出)
pub fn subtitle(text: &str) {
    if is_quiet() {
        return;
    }
    println!();
    println!(
        "{}  {}",
        colorize("▶", color::BLUE),
        colorize(text, color::BOLD)
    );
}

/// 输出信息 (quiet 模式下不输出)
pub fn info(text: &str) {
    if is_quiet() {
        return;
    }
    println!("  {}  {}", colorize("ℹ", color::BLUE), text);
}

/// 输出成功 (quiet 模式下仍然输出)
pub fn success(text: &str) {
    println!(
        "  {}  {}",
        colorize("✓", color::GREEN),
        colorize(text, color::GREEN)
    );
}

/// 输出警告 (quiet 模式下仍然输出)
pub fn warning(text: &str) {
    println!(
        "  {}  {}",
        colorize("⚠", color::YELLOW),
        colorize(text, color::YELLOW)
    );
}

/// 输出错误 (quiet 模式下仍然输出)
pub fn error(text: &str) {
    eprintln!(
        "  {}  {}",
        colorize("✗", color::RED),
        colorize(text, color::RED)
    );
}

/// API 调用追踪 - 进入 (仅 debug 模式)
/// 格式: [17:23:45.123] API → module::function(args)
pub fn debug_api_enter(module: &str, function: &str, args: &str) {
    if logger::is_debug() {
        use chrono::Local;
        let time = Local::now().format("%H:%M:%S%.3f");
        println!(
            "[{}] {} → {}::{}({})",
            colorize(&time.to_string(), color::DIM),
            colorize("API", color::MAGENTA),
            colorize(module, color::CYAN),
            colorize(function, color::CYAN),
            args
        );
    }
}

/// API 调用追踪 - 返回 (仅 debug 模式)
/// 格式: [17:23:45.456] API ← result_summary
pub fn debug_api_return(result: &str) {
    if logger::is_debug() {
        use chrono::Local;
        let time = Local::now().format("%H:%M:%S%.3f");
        println!(
            "[{}] {} ← {}",
            colorize(&time.to_string(), color::DIM),
            colorize("API", color::MAGENTA),
            colorize(result, color::GREEN)
        );
    }
}

/// API 调用追踪 - 错误 (仅 debug 模式)
pub fn debug_api_error(error: &str) {
    if logger::is_debug() {
        use chrono::Local;
        let time = Local::now().format("%H:%M:%S%.3f");
        eprintln!(
            "[{}] {} ✗ {}",
            colorize(&time.to_string(), color::DIM),
            colorize("API", color::MAGENTA),
            colorize(error, color::RED)
        );
    }
}

// ============================================================================
// 路径显示
// ============================================================================

/// 输出路径信息 (quiet 模式下不输出)
pub fn path_info(label: &str, path: &Path) {
    if is_quiet() {
        return;
    }
    println!(
        "  {}  {}: {}",
        colorize("📁", color::BLUE),
        colorize(label, color::DIM),
        path.display()
    );
}

// ============================================================================
// 表格输出
// ============================================================================

/// 简单表格行 (quiet 模式下不输出)
pub fn table_row(cols: &[(&str, usize)]) {
    if is_quiet() {
        return;
    }
    let formatted: Vec<String> = cols
        .iter()
        .map(|(text, width)| {
            let s = truncate_str(text, *width);
            // 计算实际显示宽度（中文字符占2格）
            let display_width = display_width(&s);
            let padding = width.saturating_sub(display_width);
            format!("{}{}", s, " ".repeat(padding))
        })
        .collect();
    println!("  {}", formatted.join("  "));
}

/// 表格分隔线 (quiet 模式下不输出)
pub fn table_separator(widths: &[usize]) {
    if is_quiet() {
        return;
    }
    let line: String = widths
        .iter()
        .map(|w| "─".repeat(*w))
        .collect::<Vec<_>>()
        .join("──");
    println!("  {}", colorize(&line, color::DIM));
}

/// 表格标题行 (quiet 模式下不输出)
pub fn table_header(cols: &[(&str, usize)]) {
    if is_quiet() {
        return;
    }
    let formatted: Vec<String> = cols
        .iter()
        .map(|(text, width)| format!("{:width$}", text, width = width))
        .collect();
    println!("  {}", colorize(&formatted.join("  "), color::BOLD));

    let widths: Vec<usize> = cols.iter().map(|(_, w)| *w).collect();
    table_separator(&widths);
}

// ============================================================================
// Box 输出
// ============================================================================

/// Box 宽度常量（内容区域宽度，不含边框）
const BOX_INNER_WIDTH: usize = 50;

/// 输出带边框的内容块开始 (quiet 模式下不输出)
pub fn box_start(title: &str) {
    if is_quiet() {
        return;
    }
    // 格式: ┌─ title ─────────────────────────────────────────┐
    let prefix = "┌─ ";
    let suffix = " ";
    let title_width = display_width(title);

    // 计算需要多少个 ─ 来填充
    // 总宽度 = BOX_INNER_WIDTH + 2 (左右边框各1)
    // prefix(3) + title + suffix(1) + padding + ┐(1) = BOX_INNER_WIDTH + 2
    let used = 3 + title_width + 1; // prefix 宽度 + title 宽度 + suffix 宽度
    let padding_count = (BOX_INNER_WIDTH + 2).saturating_sub(used + 1); // -1 for ┐
    let padding = "─".repeat(padding_count);

    println!(
        "{}",
        colorize(
            &format!("{}{}{}{}┐", prefix, title, suffix, padding),
            color::CYAN
        )
    );
}

/// 输出 Box 内容行 (quiet 模式下不输出)
pub fn box_line(label: &str, value: &str) {
    if is_quiet() {
        return;
    }
    // 格式: │ Label:        value                              │
    let label_col_width = 14; // label 列固定宽度

    let label_part = if label.is_empty() {
        " ".repeat(label_col_width)
    } else {
        let label_with_colon = format!("{}:", label);
        let label_width = display_width(&label_with_colon);
        let padding = " ".repeat(label_col_width.saturating_sub(label_width));
        format!("{}{}", label_with_colon, padding)
    };

    // 计算 value 的最大宽度
    let value_max_width = BOX_INNER_WIDTH.saturating_sub(label_col_width + 1); // -1 for space before │
    let truncated_value = truncate_str(value, value_max_width);
    let value_width = display_width(&truncated_value);

    // 计算右侧填充
    let content = format!("{}{}", label_part, truncated_value);
    let content_width = display_width(&label_part) + value_width;
    let right_padding = " ".repeat(BOX_INNER_WIDTH.saturating_sub(content_width));

    println!(
        "{} {}{} {}",
        colorize("│", color::CYAN),
        content,
        right_padding,
        colorize("│", color::CYAN)
    );
}

/// 输出 Box 结束行 (quiet 模式下不输出)
pub fn box_end() {
    if is_quiet() {
        return;
    }
    // 格式: └──────────────────────────────────────────────────┘
    let inner = "─".repeat(BOX_INNER_WIDTH);
    println!("{}", colorize(&format!("└{}┘", inner), color::CYAN));
}

// ============================================================================
// 进度显示
// ============================================================================

/// 简单进度条
pub fn progress_bar(current: usize, total: usize, width: usize) -> String {
    if total == 0 {
        return "░".repeat(width);
    }
    let filled = (current * width) / total;
    let empty = width - filled;
    format!(
        "{}{}",
        colorize(&"█".repeat(filled), color::GREEN),
        colorize(&"░".repeat(empty), color::DIM)
    )
}

/// 输出进度 (quiet 模式和 debug 模式下不输出)
pub fn progress(label: &str, current: usize, total: usize) {
    // quiet 模式或 debug 模式下不显示进度条
    if is_quiet() || logger::is_debug() {
        return;
    }

    // 保存进度条状态
    if let Ok(mut state) = PROGRESS_STATE.lock() {
        state.active = true;
        state.label = label.to_string();
        state.current = current;
        state.total = total;
    }

    render_progress(label, current, total);
}

/// 内部渲染进度条（不更新状态）
fn render_progress(label: &str, current: usize, total: usize) {
    let bar = progress_bar(current, total, 20);
    let percent = if total > 0 { current * 100 / total } else { 0 };
    print!(
        "\r  {}  {} [{}] {}%  ",
        colorize("⏳", color::YELLOW),
        label,
        bar,
        percent
    );
    use std::io::Write;
    let _ = std::io::stdout().flush();
}

/// 清除进度行
pub fn clear_progress() {
    // debug 模式下不操作（因为根本没有进度条）
    if is_quiet() || logger::is_debug() {
        return;
    }

    // 清除进度条状态
    if let Ok(mut state) = PROGRESS_STATE.lock() {
        state.active = false;
    }

    print!("\r{}\r", " ".repeat(100));
    use std::io::Write;
    let _ = std::io::stdout().flush();
}

// ============================================================================
// 统计输出
// ============================================================================

/// 输出统计项 (quiet 模式下不输出)
pub fn stat(label: &str, value: impl std::fmt::Display) {
    if is_quiet() {
        return;
    }
    println!(
        "  {:20} {}",
        colorize(&format!("{}:", label), color::DIM),
        colorize(&value.to_string(), color::BOLD)
    );
}

/// 输出布尔选项 (quiet 模式下不输出)
pub fn option_bool(label: &str, enabled: bool) {
    if is_quiet() {
        return;
    }
    let (icon, status) = if enabled {
        (
            colorize("✓", color::GREEN),
            colorize("enabled", color::GREEN),
        )
    } else {
        (colorize("✗", color::DIM), colorize("disabled", color::DIM))
    };
    println!(
        "  {}  {:18} {}",
        icon,
        colorize(&format!("{}:", label), color::DIM),
        status
    );
}

/// 输出执行步骤 (quiet 模式下不输出)
pub fn step(num: usize, text: &str) {
    if is_quiet() {
        return;
    }
    println!(
        "  {}  {}",
        colorize(&format!("[{}]", num), color::MAGENTA),
        text
    );
}

/// 输出带单位的大小
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

// ============================================================================
// Quiet 模式专用输出
// ============================================================================

// ============================================================================
// Quiet 模式专用输出
// ============================================================================

/// Quiet 模式下的结果输出 (始终输出)
/// 格式: Done in 45.2s (21 PKG → 156 images)
pub fn quiet_result(duration_secs: f64, pkg_count: usize, image_count: usize) {
    println!(
        "Done in {:.1}s ({} PKG → {} images)",
        duration_secs, pkg_count, image_count
    );
}

// ============================================================================
// 确认提示
// ============================================================================

/// 请求用户确认
pub fn confirm(prompt: &str) -> bool {
    use std::io::Write;
    print!("  {} {} [y/N]: ", colorize("?", color::YELLOW), prompt);
    let _ = std::io::stdout().flush();

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

/// Windows 下按任意键继续
#[cfg(windows)]
pub fn press_enter_to_exit_with_config(config_path: Option<&Path>) {
    use std::io::Write;
    if let Some(path) = config_path {
        println!("\n  配置文件路径: {}", path.display());
    }
    print!("\n  Press Enter to exit...");
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().read_line(&mut String::new());
}

#[cfg(not(windows))]
pub fn press_enter_to_exit_with_config(_config_path: Option<&Path>) {
    // Linux/macOS 不需要
}

// ============================================================================
// 特殊标记
// ============================================================================

/// PKG 标记
pub fn pkg_badge(has_pkg: bool, count: Option<usize>) -> String {
    if has_pkg {
        let text = match count {
            Some(n) => format!("✓ ({} files)", n),
            None => "✓".to_string(),
        };
        colorize(&text, color::GREEN)
    } else {
        colorize("✗", color::DIM)
    }
}

/// TEX 标记
pub fn tex_badge(is_tex: bool) -> String {
    if is_tex {
        colorize("[TEX]", color::MAGENTA)
    } else {
        String::new()
    }
}


