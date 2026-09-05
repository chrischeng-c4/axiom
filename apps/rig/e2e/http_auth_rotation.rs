use std::{
    collections::BTreeMap,
    io::{Read, Write},
    net::TcpListener,
    path::Path,
    sync::mpsc,
};

use rig::{
    engine::http,
    scenario::{
        parse_scenario,
        step::{HttpExpect, HttpRequest},
        VarStore,
    },
};

fn capture_two_requests() -> (String, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind HTTP capture server");
    let address = listener.local_addr().expect("capture server address");
    let (send, receive) = mpsc::channel();
    std::thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().expect("accept HTTP request");
            let mut request = Vec::new();
            let mut buffer = [0_u8; 4096];
            loop {
                let read = stream.read(&mut buffer).expect("read HTTP request");
                request.extend_from_slice(&buffer[..read]);
                if read == 0 {
                    break;
                }
                if let Some(header_end) = request
                    .windows(4)
                    .position(|window| window == b"\r\n\r\n")
                    .map(|position| position + 4)
                {
                    let headers = String::from_utf8_lossy(&request[..header_end]);
                    let content_length = headers
                        .lines()
                        .find_map(|line| {
                            let (name, value) = line.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().ok())
                                .flatten()
                        })
                        .unwrap_or(0);
                    if request.len() >= header_end + content_length {
                        break;
                    }
                }
            }
            send.send(String::from_utf8(request).expect("HTTP request is UTF-8"))
                .expect("publish captured request");
            stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\nconnection: close\r\n\r\n{}")
                .expect("write HTTP response");
        }
    });
    (format!("http://{address}/v1/logs"), receive)
}

#[test]
fn http_requests_forward_headers_and_reread_a_rotated_bearer_file() {
    let token_dir = tempfile::tempdir().expect("create token directory");
    let token_path = token_dir.path().join("token");
    std::fs::write(&token_path, "token-one\n").expect("write first token");
    let (url, requests) = capture_two_requests();
    let request = HttpRequest {
        method: "POST".into(),
        url,
        body: Some("{}".into()),
        headers: BTreeMap::from([("x-sift-project".into(), "sift-mvp".into())]),
        bearer_token_file: Some(token_path.display().to_string()),
        expect: HttpExpect::default(),
    };

    let first = http::execute(&request, &VarStore::new()).expect("send first request");
    assert_eq!(first.status, 200, "{first:?}");
    std::fs::write(&token_path, "token-two\n").expect("rotate token");
    let second = http::execute(&request, &VarStore::new()).expect("send second request");
    assert_eq!(second.status, 200, "{second:?}");

    let first = requests
        .recv()
        .expect("capture first request")
        .to_lowercase();
    let second = requests
        .recv()
        .expect("capture second request")
        .to_lowercase();
    assert!(
        first.contains("\r\nx-sift-project: sift-mvp\r\n"),
        "{first}"
    );
    assert!(
        first.contains("\r\nauthorization: bearer token-one\r\n"),
        "{first}"
    );
    assert!(
        second.contains("\r\nauthorization: bearer token-two\r\n"),
        "{second}"
    );
    assert!(!second.contains("token-one"), "{second}");
}

#[test]
fn gke_load_scenario_parses_project_header_and_bearer_file() {
    let scenario = parse_scenario(
        Path::new("scenarios/load/steady_logs.toml"),
        r#"
[record]
suite = "sift"
dimension = "load"
case = "steady_logs"
subject = "send logs"
kind = "load"
expected = "pass"
required = true

[load]
target_qps = 5
workers = 8
duration_secs = 1800
warmup_secs = 0

[load.request]
method = "POST"
url = "http://sift.sift.svc.cluster.local:7380/v1/logs"
bearer_token_file = "/var/run/secrets/sift/token"
body = '{}'

[load.request.headers]
x-sift-project = "sift-mvp"

[load.request.expect]
status = 200
timeout_ms = 5000
"#,
    )
    .expect("parse GKE load scenario");

    let request = &scenario.load.expect("load profile").request;
    assert_eq!(request.headers["x-sift-project"], "sift-mvp");
    assert_eq!(
        request
            .bearer_token_file
            .as_deref()
            .expect("bearer token file"),
        "/var/run/secrets/sift/token"
    );
}

#[test]
fn explicit_authorization_and_bearer_file_are_rejected_without_leaking_the_token() {
    let token_dir = tempfile::tempdir().expect("create token directory");
    let token_path = token_dir.path().join("token");
    std::fs::write(&token_path, "private-token\n").expect("write token");
    let request = HttpRequest {
        method: "GET".into(),
        url: "http://127.0.0.1:1/never-sent".into(),
        body: None,
        headers: BTreeMap::from([("Authorization".into(), "Bearer static".into())]),
        bearer_token_file: Some(token_path.display().to_string()),
        expect: HttpExpect::default(),
    };

    let error = http::execute(&request, &VarStore::new()).expect_err("reject ambiguous auth");
    assert!(error.contains("cannot set both"), "{error}");
    assert!(!error.contains("private-token"), "{error}");
}
