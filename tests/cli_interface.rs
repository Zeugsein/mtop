/// Integration tests for cli-interface spec requirements.
/// Each test cites the FR requirement from:
///   openspec/changes/archive/2026-04-05-mvp-core/specs/cli-interface/spec.md
///
/// Tests marked #[ignore] cover known PARTIAL / FAIL items in the compliance audit.

use clap::Parser;
use mtop::Cli;
use mtop::cli::{Command, TempUnit};

// ---------------------------------------------------------------------------
// FR-1: Default TUI mode
// ---------------------------------------------------------------------------

#[test]
/// FR-1: when no subcommand is given, Cli.command is None (TUI mode)
fn no_subcommand_means_tui_mode() {
    let cli = Cli::parse_from(["mtop"]);
    assert!(
        cli.command.is_none(),
        "no subcommand should produce command=None (TUI mode)"
    );
}

#[test]
/// FR-1: default interval is 1000ms when no --interval is given
fn default_interval_is_1000ms() {
    let cli = Cli::parse_from(["mtop"]);
    assert_eq!(cli.interval, 1000, "default interval should be 1000ms");
}

// ---------------------------------------------------------------------------
// FR-2: pipe subcommand
// ---------------------------------------------------------------------------

#[test]
/// FR-2: `mtop pipe` parses as Command::Pipe with samples=0 (infinite)
fn pipe_subcommand_parses() {
    let cli = Cli::parse_from(["mtop", "pipe"]);
    match cli.command {
        Some(Command::Pipe { samples }) => {
            assert_eq!(samples, 0, "default pipe samples should be 0 (infinite)");
        }
        other => panic!("expected Pipe subcommand; got {other:?}"),
    }
}

#[test]
/// FR-2: `mtop pipe --samples 10` parses samples=10
fn pipe_subcommand_samples_flag() {
    let cli = Cli::parse_from(["mtop", "pipe", "--samples", "10"]);
    match cli.command {
        Some(Command::Pipe { samples }) => {
            assert_eq!(samples, 10, "--samples 10 should set samples=10");
        }
        other => panic!("expected Pipe subcommand; got {other:?}"),
    }
}

#[test]
/// FR-2: `mtop pipe -s 5` short flag also works
fn pipe_subcommand_samples_short_flag() {
    let cli = Cli::parse_from(["mtop", "pipe", "-s", "5"]);
    match cli.command {
        Some(Command::Pipe { samples }) => {
            assert_eq!(samples, 5);
        }
        other => panic!("expected Pipe subcommand; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// FR-3: serve subcommand
// ---------------------------------------------------------------------------

#[test]
/// FR-3: `mtop serve` parses as Command::Serve with port=9090
fn serve_subcommand_default_port() {
    let cli = Cli::parse_from(["mtop", "serve"]);
    match cli.command {
        Some(Command::Serve { port }) => {
            assert_eq!(port, 9090, "default serve port should be 9090");
        }
        other => panic!("expected Serve subcommand; got {other:?}"),
    }
}

#[test]
/// FR-3: `mtop serve --port 8080` parses port=8080
fn serve_subcommand_custom_port() {
    let cli = Cli::parse_from(["mtop", "serve", "--port", "8080"]);
    match cli.command {
        Some(Command::Serve { port }) => {
            assert_eq!(port, 8080, "--port 8080 should set port=8080");
        }
        other => panic!("expected Serve subcommand; got {other:?}"),
    }
}

#[test]
/// FR-3: `mtop serve -p 7777` short flag also works
fn serve_subcommand_port_short_flag() {
    let cli = Cli::parse_from(["mtop", "serve", "-p", "7777"]);
    match cli.command {
        Some(Command::Serve { port }) => {
            assert_eq!(port, 7777);
        }
        other => panic!("expected Serve subcommand; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// FR-4: --interval / -i global option
// ---------------------------------------------------------------------------

#[test]
/// FR-4: `mtop --interval 500` sets interval=500
fn global_interval_long_flag() {
    let cli = Cli::parse_from(["mtop", "--interval", "500"]);
    assert_eq!(cli.interval, 500);
}

#[test]
/// FR-4: `mtop -i 250` sets interval=250
fn global_interval_short_flag() {
    let cli = Cli::parse_from(["mtop", "-i", "250"]);
    assert_eq!(cli.interval, 250);
}

#[test]
/// FR-4: --interval is global and works before subcommands
fn global_interval_with_subcommand() {
    let cli = Cli::parse_from(["mtop", "--interval", "300", "pipe"]);
    assert_eq!(cli.interval, 300);
    assert!(matches!(cli.command, Some(Command::Pipe { .. })));
}

// ---------------------------------------------------------------------------
// FR-5: --color option (FAIL — flag accepted but ignored)
// ---------------------------------------------------------------------------

#[test]
/// FR-5: `mtop --color blue` parses color="blue"
fn color_flag_parses() {
    let cli = Cli::parse_from(["mtop", "--color", "blue"]);
    assert_eq!(cli.color, "blue", "--color blue should set color field to 'blue'");
}

#[test]
/// FR-5: default color is "default"
fn color_default_value() {
    let cli = Cli::parse_from(["mtop"]);
    assert_eq!(cli.color, "default", "default color should be 'default'");
}

#[test]
#[ignore] // FR-5/FR-9 (FAIL): --color flag is accepted but ignored in the TUI renderer
/// FR-5: the active color theme actually changes TUI rendering when --color is set
/// This test requires TUI rendering infrastructure and cannot be unit-tested easily;
/// it serves as a marker for the missing implementation.
fn color_flag_applied_to_tui() {
    // Verified by TUI snapshot test when renderer is instrumented.
    // For now: confirm the value reaches the TUI configuration.
    let cli = Cli::parse_from(["mtop", "--color", "blue"]);
    assert_eq!(cli.color, "blue");
    // TODO: pass cli.color into TUI init and verify theme name matches
    todo!("TUI renderer does not use cli.color yet")
}

// ---------------------------------------------------------------------------
// FR-6: --temp-unit option (PARTIAL — accepts any string, no validation)
// ---------------------------------------------------------------------------

#[test]
/// FR-6: `mtop --temp-unit fahrenheit` parses correctly
fn temp_unit_fahrenheit_parses() {
    let cli = Cli::parse_from(["mtop", "--temp-unit", "fahrenheit"]);
    assert_eq!(cli.temp_unit, TempUnit::Fahrenheit);
}

#[test]
/// FR-6: `mtop --temp-unit celsius` parses correctly
fn temp_unit_celsius_parses() {
    let cli = Cli::parse_from(["mtop", "--temp-unit", "celsius"]);
    assert_eq!(cli.temp_unit, TempUnit::Celsius);
}

#[test]
/// FR-6: default temp_unit is "celsius"
fn temp_unit_default_is_celsius() {
    let cli = Cli::parse_from(["mtop"]);
    assert_eq!(cli.temp_unit, TempUnit::Celsius, "default temp-unit should be celsius");
}

#[test]
/// FR-6: `mtop --temp-unit kelvin` should be rejected (only celsius/fahrenheit are valid)
fn temp_unit_rejects_invalid_value() {
    let result = Cli::try_parse_from(["mtop", "--temp-unit", "kelvin"]);
    assert!(
        result.is_err(),
        "--temp-unit kelvin should be rejected; only 'celsius' and 'fahrenheit' are valid"
    );
}

#[test]
/// FR-6: `mtop --temp-unit garbage` should be rejected
fn temp_unit_rejects_garbage_value() {
    let result = Cli::try_parse_from(["mtop", "--temp-unit", "garbage"]);
    assert!(
        result.is_err(),
        "--temp-unit garbage should be rejected"
    );
}

// ---------------------------------------------------------------------------
// FR-7: --version / -V flag
// ---------------------------------------------------------------------------

#[test]
/// FR-7: `mtop --version` exits with success (clap handles this via process::exit)
/// We verify the version string is set on the command by checking the rendered help.
fn version_flag_is_defined() {
    // Clap uses a special built-in version action; it doesn't appear as a regular
    // argument. The simplest way to verify it's wired up is to check that the
    // command has a version string set (which triggers --version / -V support).
    let cmd = <Cli as clap::CommandFactory>::command();
    assert!(
        cmd.get_version().is_some() || cmd.get_long_version().is_some(),
        "--version / -V support requires version to be set on the command; \
         annotate #[command(version)] or set version in Cargo.toml"
    );
}

// ---------------------------------------------------------------------------
// FR-8: --help / -h flag
// ---------------------------------------------------------------------------

#[test]
/// FR-8: CLI help text mentions all subcommands: pipe, serve, debug
fn help_mentions_all_subcommands() {
    let mut cmd = <Cli as clap::CommandFactory>::command();
    let help = format!("{}", cmd.render_help());

    for subcmd in &["pipe", "serve", "debug"] {
        assert!(
            help.contains(subcmd),
            "help text should mention subcommand '{subcmd}'"
        );
    }
}

#[test]
/// FR-8: CLI help text mentions --interval global option
fn help_mentions_interval_option() {
    let mut cmd = <Cli as clap::CommandFactory>::command();
    let help = format!("{}", cmd.render_help());
    assert!(
        help.contains("interval") || help.contains("--interval"),
        "help text should document --interval option"
    );
}

#[test]
/// FR-8: CLI help text mentions --color global option
fn help_mentions_color_option() {
    let mut cmd = <Cli as clap::CommandFactory>::command();
    let help = format!("{}", cmd.render_help());
    assert!(
        help.contains("color") || help.contains("--color"),
        "help text should document --color option"
    );
}

#[test]
/// FR-8: CLI help text mentions --temp-unit global option
fn help_mentions_temp_unit_option() {
    let mut cmd = <Cli as clap::CommandFactory>::command();
    let help = format!("{}", cmd.render_help());
    assert!(
        help.contains("temp-unit") || help.contains("temp_unit"),
        "help text should document --temp-unit option"
    );
}

// ---------------------------------------------------------------------------
// FR-9: debug subcommand
// ---------------------------------------------------------------------------

#[test]
/// FR-9: `mtop debug` parses as Command::Debug
fn debug_subcommand_parses() {
    let cli = Cli::parse_from(["mtop", "debug"]);
    assert!(
        matches!(cli.command, Some(Command::Debug)),
        "expected Debug subcommand; got {:?}",
        cli.command
    );
}

#[test]
/// FR-9: debug_info() includes chip detection information
fn debug_info_contains_chip_info() {
    use mtop::metrics::Sampler;
    let sampler = Sampler::new().expect("sampler init");
    let info = sampler.debug_info();
    assert!(
        info.contains("SoC:") || info.contains("chip") || info.contains("Apple"),
        "debug_info should mention SoC/chip; got: {info}"
    );
}

#[test]
/// FR-9: debug_info() includes core count information
fn debug_info_contains_core_counts() {
    use mtop::metrics::Sampler;
    let sampler = Sampler::new().expect("sampler init");
    let info = sampler.debug_info();
    assert!(
        info.contains("core") || info.contains("Core"),
        "debug_info should mention core counts; got: {info}"
    );
}

// ---------------------------------------------------------------------------
// NDJSON pipe output format (FR-2 / api-server FR-4)
// ---------------------------------------------------------------------------

#[test]
/// FR-2 pipe: each line of pipe output is valid JSON
fn pipe_output_lines_are_valid_json() {
    use std::process::Command;
    // Run `mtop pipe --samples 3` and check each output line
    let output = Command::new(env!("CARGO_BIN_EXE_mtop"))
        .args(["pipe", "--samples", "3"])
        .output()
        .expect("failed to run mtop binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    assert_eq!(lines.len(), 3, "pipe --samples 3 should produce exactly 3 lines; got {}", lines.len());

    for (i, line) in lines.iter().enumerate() {
        serde_json::from_str::<serde_json::Value>(line)
            .unwrap_or_else(|e| panic!("line {i} is not valid JSON: {e}\n  line: {line}"));
    }
}

#[test]
/// FR-2 pipe: each NDJSON object contains the same fields as /json endpoint
fn pipe_output_contains_required_fields() {
    use std::process::Command;
    let output = Command::new(env!("CARGO_BIN_EXE_mtop"))
        .args(["pipe", "--samples", "1"])
        .output()
        .expect("failed to run mtop binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next().expect("expected at least one line");
    let json: serde_json::Value = serde_json::from_str(line).expect("line should be valid JSON");

    for field in &["timestamp", "soc", "cpu", "gpu", "power", "temperature", "memory", "network", "disk"] {
        assert!(
            json.get(field).is_some(),
            "pipe JSON object missing required field '{field}'"
        );
    }
}

#[test]
/// FR-2 pipe: pipe --samples 0 (infinite) exits when sent SIGTERM
fn pipe_samples_0_exits_on_signal() {
    use std::process::Command;
    let mut child = Command::new(env!("CARGO_BIN_EXE_mtop"))
        .args(["pipe", "--samples", "0"])
        .stdout(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn mtop pipe");

    // Give it a moment to start
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Send SIGTERM
    unsafe { libc::kill(child.id() as libc::pid_t, libc::SIGTERM) };

    let status = child.wait().expect("wait failed");
    // SIGTERM kill may give non-zero exit, but it should not hang
    let _ = status;
}
