use std::process::exit;

fn main() {
    #[cfg(feature = "cli")]
    match nups2::cli::cli() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("COMMAND FAILED:");
            eprintln!("{:?}", err);
            exit(1);
        }
    }
    #[cfg(not(feature = "cli"))]
    eprintln!("COMPILED WITHOUT CLI FEATURE -> NOTHING WILL HAPPEN");
    #[cfg(not(feature = "cli"))]
    exit(1);
}
