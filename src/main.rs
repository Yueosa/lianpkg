mod cli;

#[cfg(target_os = "windows")]
#[link(name = "resources", kind = "static")]
extern "C" {}

fn main() {
    cli::run();
}
