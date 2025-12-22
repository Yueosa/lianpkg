use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use chrono::Local;

static DEBUG_MODE: AtomicBool = AtomicBool::new(false);
static INDENT_LEVEL: AtomicUsize = AtomicUsize::new(0);

pub fn set_debug(debug: bool) {
    DEBUG_MODE.store(debug, Ordering::Relaxed);
}

pub fn is_debug() -> bool {
    DEBUG_MODE.load(Ordering::Relaxed)
}

pub fn indent() {
    INDENT_LEVEL.fetch_add(1, Ordering::Relaxed);
}

pub fn outdent() {
    INDENT_LEVEL.fetch_sub(1, Ordering::Relaxed);
}

fn get_indent_str() -> String {
    let level = INDENT_LEVEL.load(Ordering::Relaxed);
    "  ".repeat(level)
}

pub fn title(msg: &str) {
    if !is_debug() {
        println!("\n=== {} ===", msg);
    } else {
        let time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] === {} ===", time, msg);
    }
}

pub fn info(msg: &str) {
    let indent = get_indent_str();
    if !is_debug() {
        println!("{}ℹ️  {}", indent, msg);
    } else {
        let time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] [INFO] {}{}", time, indent, msg);
    }
}

pub fn success(msg: &str) {
    let indent = get_indent_str();
    if !is_debug() {
        println!("{}✅ {}", indent, msg);
    } else {
        let time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        println!("[{}] [SUCCESS] {}{}", time, indent, msg);
    }
}

pub fn error(msg: &str) {
    let indent = get_indent_str();
    if !is_debug() {
        eprintln!("{}❌ {}", indent, msg);
    } else {
        let time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        eprintln!("[{}] [ERROR] {}{}", time, indent, msg);
    }
}

pub fn debug(func_name: &str, args: &str, msg: &str) {
    if is_debug() {
        let time = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let indent = get_indent_str();
        println!("<- [DEBUG] [{}] {}[Func: {}] [Args: {}] {}", time, indent, func_name, args, msg);
    }
}

