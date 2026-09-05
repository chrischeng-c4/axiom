use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rig::engine::loadgen;
use rig::scenario::{LoadProfile, VarStore};

#[test]
fn load_stats_publish_the_mvp_p95_latency() {
    assert_eq!(loadgen::LoadStats::default().get("p95_ms"), Some(0.0));
}

fn read_request(stream: &mut TcpStream) -> String {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut request = Vec::new();
    let mut buf = [0_u8; 4096];
    let mut expected = None;
    loop {
        let read = stream.read(&mut buf).unwrap();
        assert!(read > 0, "client closed before sending the full request");
        request.extend_from_slice(&buf[..read]);
        if expected.is_none() {
            if let Some(header_end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                let headers = String::from_utf8_lossy(&request[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().unwrap())
                    })
                    .unwrap_or(0);
                expected = Some(header_end + 4 + content_length);
            }
        }
        if expected.is_some_and(|len| request.len() >= len) {
            return String::from_utf8(request).unwrap();
        }
    }
}

#[test]
fn load_requests_have_unique_sequence_and_file_backed_bearer_auth() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let received = Arc::new(Mutex::new(Vec::new()));
    let server_received = Arc::clone(&received);
    let server = std::thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while server_received.lock().unwrap().len() < 4 && Instant::now() < deadline {
            let (mut stream, _) = match listener.accept() {
                Ok(connection) => connection,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                    continue;
                }
                Err(error) => panic!("accept failed: {error}"),
            };
            let request = read_request(&mut stream);
            server_received.lock().unwrap().push(request);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\n{}")
                .unwrap();
        }
    });

    let token_dir = tempfile::tempdir().unwrap();
    let token_path = token_dir.path().join("token");
    std::fs::write(&token_path, "project-token\n").unwrap();
    let profile: LoadProfile = toml::from_str(&format!(
        r#"
target_qps = 4
workers = 2
duration_secs = 1

[request]
method = "POST"
url = "http://{address}/v1/logs"
body = '{{"item_id":"load-{{{{rig.sequence}}}}","padded":"{{{{rig.sequence_06}}}}","hex":"{{{{rig.sequence_016x}}}}"}}'
bearer_token_file = "{}"
headers = {{ x-sift-project = "mvp-project" }}

[request.expect.jsonpath]
"$.partialSuccess" = "absent"
"#,
        token_path.display()
    ))
    .unwrap();

    let stats = loadgen::run(&profile, &VarStore::new());
    server.join().unwrap();

    assert_eq!(stats.total, 4);
    assert_eq!(stats.failed, 0);
    let requests = received.lock().unwrap();
    let mut bodies = Vec::new();
    for request in requests.iter() {
        let lower = request.to_ascii_lowercase();
        assert!(lower.contains("authorization: bearer project-token\r\n"));
        assert!(lower.contains("x-sift-project: mvp-project\r\n"));
        bodies.push(request.split("\r\n\r\n").nth(1).unwrap().to_string());
    }
    bodies.sort();
    bodies.dedup();
    assert_eq!(
        bodies.len(),
        4,
        "each scheduled operation needs a unique body"
    );
    let mut padded = bodies
        .iter()
        .map(|body| {
            let body: serde_json::Value = serde_json::from_str(body).unwrap();
            (
                body["padded"].as_str().unwrap().to_string(),
                body["hex"].as_str().unwrap().to_string(),
            )
        })
        .collect::<Vec<_>>();
    padded.sort();
    assert_eq!(
        padded,
        vec![
            ("000000".into(), "0000000000000000".into()),
            ("000001".into(), "0000000000000001".into()),
            ("000002".into(), "0000000000000002".into()),
            ("000003".into(), "0000000000000003".into()),
        ]
    );
}

#[test]
fn load_marks_an_otlp_partial_success_as_failed() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let _ = read_request(&mut stream);
        let body = r#"{"partialSuccess":{"rejectedLogRecords":1}}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
    });
    let profile: LoadProfile = toml::from_str(&format!(
        r#"
target_qps = 1
workers = 1
duration_secs = 1

[request]
method = "POST"
url = "http://{address}/v1/logs"
body = '{{}}'

[request.expect.jsonpath]
"$.partialSuccess" = "absent"
"#
    ))
    .unwrap();

    let stats = loadgen::run(&profile, &VarStore::new());
    server.join().unwrap();
    assert_eq!(stats.total, 1);
    assert_eq!(stats.failed, 1);
}
