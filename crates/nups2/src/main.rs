use std::process::exit;

fn main() {
    //let p = std::path::PathBuf::from(std::env::args().collect::<Vec<String>>().get(1).unwrap());
    //let mut br = std::fs::File::open(&p).unwrap();
    //let fsb = nups2::fsb5::Fsb5::new(&mut br).unwrap();
    //dbg!(&fsb);
    //for sample in fsb.samples {
    //    //dbg!(sample.read(&mut br).unwrap());
    //}

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
