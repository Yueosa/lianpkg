use std::sync::atomic::{AtomicBool, Ordering};

static DEBUG_MODE: AtomicBool = AtomicBool::new(false);
static QUIET_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_debug(debug: bool) {
    DEBUG_MODE.store(debug, Ordering::Relaxed);
}

pub fn is_debug() -> bool {
    DEBUG_MODE.load(Ordering::Relaxed)
}

pub fn set_quiet(quiet: bool) {
    QUIET_MODE.store(quiet, Ordering::Relaxed);
}

pub fn is_quiet() -> bool {
    QUIET_MODE.load(Ordering::Relaxed)
}
