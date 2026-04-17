use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum TempUnit {
    Celsius,
    Fahrenheit,
}

impl std::fmt::Display for TempUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TempUnit::Celsius => write!(f, "celsius"),
            TempUnit::Fahrenheit => write!(f, "fahrenheit"),
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "mtop", version, about = "System monitor for macOS")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Update interval in milliseconds
    #[arg(short, long, global = true)]
    pub interval: Option<u32>,

    /// Color theme name
    #[arg(long, global = true)]
    pub color: Option<String>,

    /// Temperature unit: celsius or fahrenheit
    #[arg(long, global = true, value_enum, default_value_t = TempUnit::Celsius)]
    pub temp_unit: TempUnit,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Output metrics as NDJSON to stdout
    Pipe {
        /// Number of samples (0 = infinite)
        #[arg(short, long, default_value_t = 0)]
        samples: u64,
    },

    /// Start HTTP API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value_t = 9090)]
        port: u16,

        /// Address to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
        bind: String,

        /// Allow binding to external (non-loopback) interfaces (security risk)
        #[arg(long, default_value_t = false)]
        allow_external_bind: bool,

        /// Stop sampling when no requests have arrived for this many seconds (default: 30)
        #[arg(long, default_value_t = 30)]
        serve_idle_timeout: u64,

        /// Require a bearer token for all requests (auto-generates if MTOP_SERVE_TOKEN is unset)
        #[arg(long, default_value_t = false)]
        require_token: bool,
    },

    /// Print debug/diagnostic information
    Debug,
}
