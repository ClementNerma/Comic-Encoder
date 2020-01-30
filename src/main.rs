#![forbid(unsafe_code)]
#![deny(unused_must_use)]

#[macro_use] extern crate log;

mod cli;
mod lib;
mod logger;

fn main() {
    use std::time::Instant;
    use clap::{load_yaml, App};
    use log::LevelFilter;
    use cli::error::GlobalError;

    let started = Instant::now();

    macro_rules! wrap { ($result: expr) => { $result.map_err(|err| format!("{}", err)) } }

    let yaml = load_yaml!("cli/clap.yml");
    let matches = App::from_yaml(yaml).get_matches();

    logger::start(
        if matches.is_present("silent") { LevelFilter::Error } else if matches.is_present("verbose") { LevelFilter::Debug }
        else if matches.is_present("debug") { LevelFilter::Trace } else { LevelFilter::Info }
    );

    let result = match matches.subcommand() {
        ("encode", Some(args)) => wrap!(cli::encode::from_args(args)),
        ("decode", Some(args)) => wrap!(cli::decode::from_args(args)),
        ("rebuild", Some(args)) => wrap!(cli::rebuild::from_args(args)).map(|path| vec![path]),
        ("", _) => wrap!(Err(GlobalError::ActionNameIsMissing)),
        (cmd, _) => wrap!(Err(GlobalError::UnknownAction(cmd.to_owned())))
    };

    match result {
        Ok(_) => {
            let elapsed = started.elapsed();
            let secs = elapsed.as_secs();
            info!("Done in {}m{: >2}.{:03}s.", secs / 60, secs % 60, elapsed.subsec_millis());
        },

        Err(err) => {
            error!("{}", err);
            std::process::exit(1);
        }
    }
}