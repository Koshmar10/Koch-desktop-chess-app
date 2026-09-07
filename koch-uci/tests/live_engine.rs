//! Integration tests against a real, locally installed `stockfish` binary.
//! `UciParser`'s unit tests (in `src/parser.rs`) cover protocol-shape edge
//! cases from recorded fixtures with no subprocess involved — these instead
//! check that `UciProcess`/`Engine` actually drive a live process correctly
//! (spawn, handshake, search, stop, quit). If `stockfish` isn't on `PATH`,
//! each test skips with a message rather than failing, since the M4
//! bundled-vs-prerequisite decision isn't made yet — see
//! `docs/stockfish/licensing-and-distribution.md`.
//!
//! Uses a manual `Runtime::block_on` rather than `#[tokio::test]` — see the
//! comment on the `tokio` dependency in `Cargo.toml` for why.

use koch_uci::{Engine, GoLimits, SearchEvent};

fn block_on<F: std::future::Future>(fut: F) -> F::Output {
    tokio::runtime::Runtime::new()
        .expect("failed to build a tokio runtime for the test")
        .block_on(fut)
}

async fn spawn_or_skip() -> Option<Engine> {
    match Engine::spawn("stockfish").await {
        Ok(engine) => Some(engine),
        Err(err) => {
            eprintln!("skipping: no `stockfish` on PATH ({err})");
            None
        }
    }
}

#[test]
fn handshake_reports_identity_and_options() {
    block_on(async {
        let Some(engine) = spawn_or_skip().await else {
            return;
        };
        assert!(engine
            .name
            .as_deref()
            .unwrap_or_default()
            .starts_with("Stockfish"));
        assert!(engine.author.is_some());
        assert!(engine.options.iter().any(|opt| opt.name == "Threads"));
    });
}

#[test]
fn isready_responds_even_though_nothing_is_searching() {
    block_on(async {
        let Some(mut engine) = spawn_or_skip().await else {
            return;
        };
        engine.is_ready().await.expect("isready should get readyok");
    });
}

#[test]
fn a_depth_limited_search_produces_info_then_one_bestmove() {
    block_on(async {
        let Some(mut engine) = spawn_or_skip().await else {
            return;
        };
        engine.new_game().await.unwrap();
        engine.set_position_startpos(&[]).await.unwrap();

        let limits = GoLimits {
            depth: Some(4),
            ..Default::default()
        };
        engine.go(&limits).await.unwrap();

        let mut saw_info = false;
        loop {
            match engine.next_search_event().await.unwrap() {
                SearchEvent::Info(_) => saw_info = true,
                SearchEvent::BestMove { mv, .. } => {
                    assert!(mv.len() >= 4, "expected a UCI move like e2e4, got {mv:?}");
                    break;
                }
            }
        }
        assert!(saw_info, "expected at least one info line before bestmove");

        engine.quit().await.unwrap();
    });
}

#[test]
fn stop_still_yields_exactly_one_bestmove() {
    block_on(async {
        let Some(mut engine) = spawn_or_skip().await else {
            return;
        };
        engine.set_position_startpos(&[]).await.unwrap();

        let limits = GoLimits {
            infinite: true,
            ..Default::default()
        };
        engine.go(&limits).await.unwrap();

        // Let it think for a moment, then cut it short — mirrors what
        // `Engine::go_until` does internally on cancellation.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        engine.stop().await.unwrap();

        loop {
            if let SearchEvent::BestMove { .. } = engine.next_search_event().await.unwrap() {
                break;
            }
        }
    });
}
