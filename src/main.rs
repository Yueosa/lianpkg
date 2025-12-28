mod cli;

#[cfg(target_os = "windows")]
#[link(name = "resources", kind = "static")]
extern "C" {}

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
