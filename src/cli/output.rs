//! 格式化输出模块
//!
//! 提供美化的终端输出，支持表格、颜色、Box 等

use std::path::Path;

// ============================================================================
// 字符串工具
// ============================================================================

/// 计算字符串的显示宽度（中文字符占2格）
fn display_width(s: &str) -> usize {
    s.chars().map(|c| {
        if c.is_ascii() {
            1
        } else {
            // CJK 字符通常占 2 格
            2
        }
    }).sum()
}

/// 按显示宽度截断字符串（UTF-8 安全）
fn truncate_str(s: &str, max_width: usize) -> String {
    if max_width < 4 {
        return "...".to_string();
    }
    
    let mut width = 0;
    let mut result = String::new();
    
    for c in s.chars() {
        let char_width = if c.is_ascii() { 1 } else { 2 };
        if width + char_width > max_width - 3 {
            result.push_str("...");
            return result;
        }
        width += char_width;
        result.push(c);
    }
    
    result
}

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

/// 输出标题
pub fn title(text: &str) {
    let line = "═".repeat(text.len() + 4);
    println!();
    println!("{}", colorize(&line, color::CYAN));
    println!("{}", colorize(&format!("  {}  ", text), &format!("{}{}", color::BOLD, color::CYAN)));
    println!("{}", colorize(&line, color::CYAN));
}

/// 输出子标题
pub fn subtitle(text: &str) {
    println!();
    println!("{} {}", colorize("▶", color::BLUE), colorize(text, color::BOLD));
}

/// 输出信息
pub fn info(text: &str) {
    println!("  {} {}", colorize("ℹ", color::BLUE), text);
}

/// 输出成功
pub fn success(text: &str) {
    println!("  {} {}", colorize("✓", color::GREEN), colorize(text, color::GREEN));
}

/// 输出警告
pub fn warning(text: &str) {
    println!("  {} {}", colorize("⚠", color::YELLOW), colorize(text, color::YELLOW));
}

/// 输出错误
pub fn error(text: &str) {
    eprintln!("  {} {}", colorize("✗", color::RED), colorize(text, color::RED));
}

/// 输出调试信息
#[allow(dead_code)]
pub fn debug(text: &str) {
    println!("  {} {}", colorize("⋯", color::DIM), colorize(text, color::DIM));
}

// ============================================================================
// 路径显示
// ============================================================================

/// 格式化路径显示（截断过长路径）
#[allow(dead_code)]
pub fn format_path(path: &Path, max_len: usize) -> String {
    let s = path.display().to_string();
    if s.len() <= max_len {
        s
    } else {
        format!("...{}", &s[s.len() - max_len + 3..])
    }
}

/// 输出路径信息
pub fn path_info(label: &str, path: &Path) {
    println!("  {} {}: {}", 
        colorize("📁", color::BLUE),
        colorize(label, color::DIM),
        path.display()
    );
}

// ============================================================================
// 表格输出
// ============================================================================

/// 简单表格行
pub fn table_row(cols: &[(&str, usize)]) {
    let formatted: Vec<String> = cols.iter()
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

/// 表格分隔线
pub fn table_separator(widths: &[usize]) {
    let line: String = widths.iter()
        .map(|w| "─".repeat(*w))
        .collect::<Vec<_>>()
        .join("──");
    println!("  {}", colorize(&line, color::DIM));
}

/// 表格标题行
pub fn table_header(cols: &[(&str, usize)]) {
    let formatted: Vec<String> = cols.iter()
        .map(|(text, width)| format!("{:width$}", text, width = width))
        .collect();
    println!("  {}", colorize(&formatted.join("  "), color::BOLD));
    
    let widths: Vec<usize> = cols.iter().map(|(_, w)| *w).collect();
    table_separator(&widths);
}

// ============================================================================
// Box 输出
// ============================================================================

/// 输出带边框的内容块
pub fn box_start(title: &str) {
    const BOX_WIDTH: usize = 52;
    let border = format!("┌─ {} ", title);
    let border_width = display_width(&border);
    let padding = "─".repeat(BOX_WIDTH.saturating_sub(border_width));
    println!("{}{}{}", colorize(&border, color::CYAN), colorize(&padding, color::CYAN), colorize("┐", color::CYAN));
}

pub fn box_line(label: &str, value: &str) {
    const BOX_WIDTH: usize = 52;
    
    let label_part = if label.is_empty() {
        "               ".to_string()  // 15 spaces for alignment
    } else {
        format!("{:12}  ", format!("{}:", label))
    };
    
    let max_value_width = BOX_WIDTH.saturating_sub(display_width(&label_part) + 4); // 4 = "│ " + " │"
    let truncated_value = truncate_str(value, max_value_width);
    
    let content = format!("{}{}", label_part, truncated_value);
    let content_width = display_width(&content);
    let padding = " ".repeat(BOX_WIDTH.saturating_sub(content_width + 4));
    
    println!("{} {}{} {}", 
        colorize("│", color::CYAN),
        content,
        padding,
        colorize("│", color::CYAN)
    );
}

pub fn box_end() {
    println!("{}", colorize(&format!("└{}┘", "─".repeat(52)), color::CYAN));
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
    format!("{}{}",
        colorize(&"█".repeat(filled), color::GREEN),
        colorize(&"░".repeat(empty), color::DIM)
    )
}

/// 输出进度
pub fn progress(label: &str, current: usize, total: usize) {
    let bar = progress_bar(current, total, 20);
    let percent = if total > 0 { current * 100 / total } else { 0 };
    print!("\r  {} {} [{}] {}%  ", 
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
    print!("\r{}\r", " ".repeat(80));
    use std::io::Write;
    let _ = std::io::stdout().flush();
}

// ============================================================================
// 统计输出
// ============================================================================

/// 输出统计项
pub fn stat(label: &str, value: impl std::fmt::Display) {
    println!("  {:20} {}", 
        colorize(&format!("{}:", label), color::DIM),
        colorize(&value.to_string(), color::BOLD)
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
pub fn press_enter_to_exit() {
    use std::io::Write;
    use std::env;
    if let Ok(appdata) = env::var("APPDATA") {
        println!("\n  配置文件已生成于: {}\\lianpkg\\config.toml", appdata);
    } else {
        println!("\n  配置文件已生成于: %APPDATA%\\lianpkg\\config.toml");
    }
    print!("\n  Press Enter to exit...");
    let _ = std::io::stdout().flush();
    let _ = std::io::stdin().read_line(&mut String::new());
}

#[cfg(not(windows))]
pub fn press_enter_to_exit() {
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

/// 类型标记
#[allow(dead_code)]
pub fn type_badge(wallpaper_type: &str) -> String {
    match wallpaper_type.to_lowercase().as_str() {
        "scene" => colorize("scene", color::GREEN),
        "video" => colorize("video", color::BLUE),
        "web" => colorize("web", color::YELLOW),
        "preset" => colorize("preset", color::MAGENTA),
        _ => wallpaper_type.to_string(),
    }
}
