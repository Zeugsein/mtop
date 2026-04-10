use clap::Parser;
use mtop::cli::{Cli, Command};
use mtop::metrics::Sampler;
use mtop::{config, serve, tui};
use std::sync::{Arc, RwLock};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg = config::load();
    let args = Cli::parse();

    // CLI args override config values (clap defaults are distinguishable via matches)
    let interval = if args.interval != 1000 { args.interval } else { cfg.interval_ms };
    let color = if args.color != "default" { args.color.clone() } else { cfg.theme.clone() };
    let temp_unit = args.temp_unit.to_string();

    match args.command {
        Some(Command::Pipe { samples }) => {
            let mut sampler = Sampler::new()?;
            let mut count = 0u64;

            loop {
                let snapshot = sampler.sample(interval)?;
                let json = serde_json::to_string(&snapshot)?;
                println!("{json}");

                count += 1;
                if samples > 0 && count >= samples {
                    break;
                }
            }
        }
        Some(Command::Serve { port, bind }) => {
            let mut sampler = Sampler::new()?;
            let soc = sampler.soc_info().clone();
            let shared = Arc::new(RwLock::new(None));

            let shared_http = Arc::clone(&shared);
            std::thread::spawn(move || {
                if let Err(e) = serve::run(port, &bind, shared_http, &soc) {
                    eprintln!("server error: {e}");
                }
            });

            loop {
                match sampler.sample(interval) {
                    Ok(s) => {
                        if let Ok(mut guard) = shared.write() {
                            *guard = Some(s);
                        } else {
                            eprintln!("metrics lock poisoned, skipping update");
                        }
                    }
                    Err(e) => eprintln!("sampling error: {e}"),
                }
            }
        }
        Some(Command::Debug) => {
            let sampler = Sampler::new()?;
            println!("{}", sampler.debug_info());
        }
        None => {
            tui::run(interval, &color, &temp_unit)?;
        }
    }

    Ok(())
}
