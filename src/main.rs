use clap::Parser;
use mtop::cli::{Cli, Command};
use mtop::metrics::Sampler;
use mtop::{config, serve, tui};
use parking_lot::{Condvar, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

fn is_loopback(addr: &str) -> bool {
    if addr.starts_with("127.") {
        return true;
    }
    if addr == "::1" {
        return true;
    }
    if addr == "localhost" {
        return true;
    }
    false
}

/// Parse env var as a truthy boolean (1, true, yes).
fn env_bool(key: &str) -> bool {
    matches!(
        std::env::var(key)
            .unwrap_or_default()
            .to_lowercase()
            .as_str(),
        "1" | "true" | "yes"
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // SHALL-52-ENV-2: load .env files before Cli::parse() and config::load()
    config::load_dotenv();

    let cfg = config::load();
    let args = Cli::parse();

    // CLI args override config values; None means "not specified, use config"
    let interval = args.interval.unwrap_or(cfg.interval_ms);
    let color = args.color.unwrap_or_else(|| cfg.theme.clone());
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
        Some(Command::Serve {
            port,
            bind,
            allow_external_bind,
            serve_idle_timeout,
            require_token,
        }) => {
            // SHALL-52-S2-2: env var overrides CLI flag (if CLI is false, check env)
            let allow_external_bind = allow_external_bind || env_bool("MTOP_ALLOW_EXTERNAL_BIND");
            // SHALL-52-S4-1: env var override for idle timeout
            let idle_timeout_secs: u64 = std::env::var("MTOP_SERVE_IDLE_TIMEOUT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(serve_idle_timeout);
            // SHALL-52-S5-1b: env var override for require_token
            let require_token = require_token || env_bool("MTOP_REQUIRE_TOKEN");

            // SHALL-52-S2-3: reject external bind unless opted in
            if !is_loopback(&bind) && !allow_external_bind {
                eprintln!("error: mtop serve refuses to bind to {bind} (external interface)");
                eprintln!(
                    "       metrics are exposed without authentication — this is a security risk"
                );
                eprintln!(
                    "       to opt in explicitly: --allow-external-bind  |  MTOP_ALLOW_EXTERNAL_BIND=1  |  .env: MTOP_ALLOW_EXTERNAL_BIND=1"
                );
                std::process::exit(1);
            }

            // SHALL-52-S5-2: resolve bearer token
            let token: Option<String> = resolve_token(require_token)?;

            // SHALL-52-S5-5: warn when external bind + open mode
            if allow_external_bind && token.is_none() {
                eprintln!("warning: mtop serve is bound to {bind} without authentication");
                eprintln!(
                    "         consider setting a token: use --require-token (auto-generates) or set MTOP_SERVE_TOKEN"
                );
            }

            let mut sampler = Sampler::new()?;
            let soc = sampler.soc_info().clone();
            let shared = Arc::new(RwLock::new(None));

            // SHALL-52-S4-2: shared last-request timestamp
            let last_request: Arc<AtomicU64> = Arc::new(AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            ));

            // ADR-0014: parking_lot::Condvar pairs for idle-resume signaling
            // collect_now: serve → main ("collect immediately")
            // collect_done: main → serve ("fresh data ready", generation counter)
            let collect_now: Arc<(Mutex<bool>, Condvar)> =
                Arc::new((Mutex::new(false), Condvar::new()));
            let collect_done: Arc<(Mutex<u64>, Condvar)> =
                Arc::new((Mutex::new(0u64), Condvar::new()));

            let shared_http = Arc::clone(&shared);
            let last_request_serve = Arc::clone(&last_request);
            let cn_serve = Arc::clone(&collect_now);
            let cd_serve = Arc::clone(&collect_done);
            std::thread::spawn(move || {
                if let Err(e) = serve::run(
                    port,
                    &bind,
                    shared_http,
                    &soc,
                    last_request_serve,
                    idle_timeout_secs,
                    cn_serve,
                    cd_serve,
                    token,
                ) {
                    eprintln!("server error: {e}");
                }
            });

            loop {
                // SHALL-52-S4-4: idle check — park on condvar when idle
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let last = last_request.load(Ordering::Relaxed);
                if now.saturating_sub(last) > idle_timeout_secs {
                    // Park until collect_now signal or interval timeout
                    let mut flag = collect_now.0.lock();
                    if !*flag {
                        collect_now
                            .1
                            .wait_for(&mut flag, Duration::from_millis(interval as u64));
                    }
                    if *flag {
                        // Serve thread requested immediate collection on idle-resume
                        *flag = false;
                        drop(flag);
                        match sampler.sample(100) {
                            Ok(s) => {
                                if let Ok(mut guard) = shared.write() {
                                    *guard = Some(s);
                                }
                                let mut coll_gen = collect_done.0.lock();
                                *coll_gen += 1;
                                collect_done.1.notify_all();
                            }
                            Err(e) => eprintln!("sampling error (idle-resume): {e}"),
                        }
                    }
                    continue;
                }

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

/// SHALL-52-S5-2: Resolve the bearer token.
/// Order: MTOP_SERVE_TOKEN env var → auto-generate if require_token → None (open mode)
fn resolve_token(require_token: bool) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Step 2: check if already set
    if let Ok(t) = std::env::var("MTOP_SERVE_TOKEN")
        && !t.is_empty()
    {
        return Ok(Some(t));
    }

    // Step 3: auto-generate if required
    if require_token {
        let token = generate_token()?;
        println!("mtop: generated bearer token (saved to ~/.mtop/.env):");
        println!("MTOP_SERVE_TOKEN={token}");

        // Write to ~/.mtop/.env
        write_token_to_env_file(&token)?;

        // Set in process environment for serve::run to use
        #[allow(deprecated)]
        unsafe {
            std::env::set_var("MTOP_SERVE_TOKEN", &token)
        };

        return Ok(Some(token));
    }

    // Open mode
    Ok(None)
}

/// Read 32 bytes from /dev/urandom and hex-encode them.
fn generate_token() -> Result<String, Box<dyn std::error::Error>> {
    use std::io::Read as _;
    let mut f = std::fs::File::open("/dev/urandom")?;
    let mut bytes = [0u8; 32];
    f.read_exact(&mut bytes)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// Write or replace MTOP_SERVE_TOKEN in ~/.mtop/.env.
fn write_token_to_env_file(token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let dir = std::path::PathBuf::from(home).join(".mtop");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(".env");

    let line = format!("MTOP_SERVE_TOKEN={token}");

    // If file exists, replace existing MTOP_SERVE_TOKEN line or append
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let mut found = false;
    let updated: String = existing
        .lines()
        .map(|l| {
            if l.starts_with("MTOP_SERVE_TOKEN=") {
                found = true;
                line.clone()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let content = if found {
        if updated.is_empty() {
            updated
        } else {
            format!("{updated}\n")
        }
    } else if existing.is_empty() {
        format!("{line}\n")
    } else {
        // Ensure existing content ends with newline before appending
        let sep = if existing.ends_with('\n') { "" } else { "\n" };
        format!("{existing}{sep}{line}\n")
    };

    std::fs::write(&path, content)?;
    Ok(())
}
