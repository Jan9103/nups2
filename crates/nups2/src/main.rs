use std::process::exit;

fn main() {
    start();
}
#[cfg(feature = "cli")]
fn start() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::debug!("Debug log visible");
    log::trace!("Trace log visible");

    match nups2::cli::cli() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("COMMAND FAILED:");
            eprintln!("{}", err);
            exit(1);
        }
    }
}

#[cfg(not(feature = "cli"))]
fn start() {
    eprintln!("COMPILED WITHOUT CLI FEATURE -> NOTHING WILL HAPPEN");
    exit(1);
}
