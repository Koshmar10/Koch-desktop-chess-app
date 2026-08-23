//! Manual testing tool for `koch-uci`: spawns an engine and lets you type
//! raw UCI commands at it, printing every reply raw *and* parsed. Exists
//! because testing a spawned process purely through `cargo test` makes it
//! hard to just poke at the thing interactively — this is that poke tool.
//!
//! Usage: `cargo run -p koch-uci --bin uci-cli [path-to-engine]`
//! (defaults to `stockfish` on `PATH`). Type UCI commands, e.g.:
//!   uci
//!   isready
//!   position startpos
//!   go depth 10
//!   stop
//!   quit

use std::env;

use koch_uci::parser::{parse_line, UciMessage};
use koch_uci::UciProcess;
use tokio::io::{AsyncBufReadExt, BufReader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::runtime::Runtime::new()?.block_on(run())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let engine_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "stockfish".to_string());
    eprintln!("spawning `{engine_path}` — type raw UCI commands, Ctrl-D or `quit` to exit");

    let process = UciProcess::spawn(&engine_path).await?;
    // Split so reading engine output and reading your typing can run on
    // separate tasks — no need to race them against each other.
    let (mut writer, mut reader) = process.split();

    let printer = tokio::spawn(async move {
        while let Some(raw) = reader.next_line().await {
            println!("< {raw}");
            let parsed = parse_line(&raw);
            if !matches!(parsed, UciMessage::Ignored) {
                println!("  parsed: {parsed:?}");
            }
        }
        eprintln!("engine process exited");
    });

    let mut stdin_lines = BufReader::new(tokio::io::stdin()).lines();
    while let Some(cmd) = stdin_lines.next_line().await? {
        let quit = cmd.trim() == "quit";
        writer.write_line(&cmd).await?;
        if quit {
            break;
        }
    }

    let _ = printer.await;
    Ok(())
}
