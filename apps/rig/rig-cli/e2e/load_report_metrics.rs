use std::io::{Read, Write};
use std::net::TcpListener;

use clap::Parser;
use rig_cli::dispatch::{execute, RigCommand};

#[test]
fn a_clean_load_report_keeps_latency_and_count_evidence() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0_u8; 2048];
            let _ = stream.read(&mut request);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\n{}")
                .unwrap();
        }
    });

    let root = tempfile::tempdir().unwrap();
    let load_dir = root.path().join("load");
    std::fs::create_dir(&load_dir).unwrap();
    let scenario = load_dir.join("http_observation.toml");
    std::fs::write(
        &scenario,
        format!(
            r#"
[record]
suite = "rig"
dimension = "load"
case = "http_observation"
subject = "clean load evidence remains visible"
kind = "load"
expected = "pass"

[load]
target_qps = 2
workers = 1
duration_secs = 1

[load.request]
method = "GET"
url = "http://{address}/readyz"
"#
        ),
    )
    .unwrap();

    let command = RigCommand::parse_from(["rig", "run", "--scenario", scenario.to_str().unwrap()]);
    let report = execute(command);
    server.join().unwrap();
    assert!(report.clean);
    let json = serde_json::to_value(report).unwrap();
    let evidence = json["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|finding| finding["kind"] == "load_observation")
        .map(|finding| &finding["evidence"])
        .expect("clean load report must retain one load_observation");
    assert_eq!(evidence["total"], 2);
    assert_eq!(evidence["failed"], 0);
    assert!(evidence["p95_ms"].as_f64().unwrap() > 0.0);
}
