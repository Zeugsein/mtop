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
    #[arg(short, long, global = true, default_value_t = 1000)]
    pub interval: u32,

    /// Color theme name
    #[arg(long, global = true, default_value = "default")]
    pub color: String,

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
        samples: u32,
    },

    /// Start HTTP API server
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value_t = 9090)]
        port: u16,

        /// Address to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
        bind: String,
    },

    /// Print debug/diagnostic information
    Debug,
}
