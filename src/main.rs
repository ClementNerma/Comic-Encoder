#![forbid(unsafe_code)]
#![deny(unused_must_use)]

#[macro_use] extern crate log;

pub mod lib;
pub mod cli;

mod actions;
mod logger;

use std::time::Instant;
use log::LevelFilter;
use clap::Clap;
use cli::opts::{Opts, Action, EncodingMethod};

fn main() {
    let started = Instant::now();

    let opts: Opts = Opts::parse();

    logger::start(
        if opts.silent { LevelFilter::Error } else if opts.verbose { LevelFilter::Debug }
        else if opts.debug { LevelFilter::Trace } else { LevelFilter::Info }
    );

    trace!("Command-line arguments were parsed successfully.");

    let result = match &opts.action {
        Action::Encode(opts) => match &opts.method {
            EncodingMethod::Compile(compile_opts) =>
                actions::compile(compile_opts, &opts.options)
                    .map_err(|err| format!("{}", err)),

            EncodingMethod::Single(one_opts) =>
                actions::encode_one(one_opts, &opts.options)
                    .map(|path| vec![path])
                    .map_err(|err| format!("{}", err)),
        },

        Action::Decode(decode) =>
            actions::decode(decode)
                .map_err(|err| format!("{}", err))
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
