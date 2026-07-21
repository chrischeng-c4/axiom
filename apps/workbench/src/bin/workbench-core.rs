// HANDWRITE-BEGIN gap="missing-generator:logic:e4006843" tracker="pending-tracker" reason="Run one stdin/stdout sidecar loop with JSON only on stdout, ordered responses, explicit shutdown, and terminal cleanup."
use std::io::{self, BufRead, BufWriter, Write};

use workbench::core_protocol::ProtocolServer;

fn main() {
    if let Err(error) = run() {
        eprintln!("workbench-core: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let stdin = io::stdin();
    let mut stdout = BufWriter::new(io::stdout().lock());
    let mut server = ProtocolServer::default();

    for line in stdin.lock().lines() {
        let line = line.map_err(|error| format!("read request: {error}"))?;
        if line.trim().is_empty() {
            continue;
        }
        let outcome = server.handle_line(&line);
        serde_json::to_writer(&mut stdout, &outcome.response)
            .map_err(|error| format!("encode response: {error}"))?;
        stdout
            .write_all(b"\n")
            .and_then(|_| stdout.flush())
            .map_err(|error| format!("write response: {error}"))?;
        if outcome.shutdown {
            break;
        }
    }

    server.terminate_all();
    Ok(())
}

<!-- marker: missing-generator:logic:e4006843 path: apps/workbench/src/bin/workbench-core.rs reason: Run one stdin/stdout sidecar loop with JSON only on stdout, ordered responses, explicit shutdown, and terminal cleanup. -->
// HANDWRITE-END
