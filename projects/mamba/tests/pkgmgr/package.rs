//! CLI integration tests for `mamba package` and `mamba publish`.

use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("spawn mamba")
}

fn write_project(dir: &Path) {
    std::fs::create_dir_all(dir.join("src/demo_pkg")).unwrap();
    std::fs::write(
        dir.join("pyproject.toml"),
        r#"[project]
name = "demo-pkg"
version = "0.1.0"
description = "Demo package"
requires-python = ">=3.10"
dependencies = ["requests>=2"]

[project.optional-dependencies]
dev = ["pytest>=8"]
"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("src/demo_pkg/__init__.py"),
        "__version__ = '0.1.0'\n",
    )
    .unwrap();
}

fn read_wheel_entry(path: &Path, entry: &str) -> String {
    let file = File::open(path).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let mut entry = zip.by_name(entry).unwrap();
    let mut body = String::new();
    entry.read_to_string(&mut body).unwrap();
    body
}

fn read_sdist_entry(path: &Path, suffix: &str) -> String {
    let file = File::open(path).unwrap();
    let gz = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(gz);
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        let entry_path = entry.path().unwrap().to_string_lossy().replace('\\', "/");
        if entry_path.ends_with(suffix) {
            let mut body = String::new();
            entry.read_to_string(&mut body).unwrap();
            return body;
        }
    }
    panic!("missing sdist entry suffix {suffix}");
}

fn spawn_publish_server(
    expected_auth: String,
    status: u16,
    response_body: &'static str,
) -> (String, JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(10);
        let (mut stream, _) = loop {
            match listener.accept() {
                Ok(pair) => break pair,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        panic!("timed out waiting for publish upload");
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(e) => panic!("accept publish upload: {e}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        let request = read_http_request(&mut stream);
        let text = String::from_utf8_lossy(&request);
        let lower = text.to_ascii_lowercase();
        assert!(text.starts_with("POST /legacy/ HTTP/1.1"), "{text}");
        assert!(
            lower.contains(&format!(
                "authorization: {}",
                expected_auth.to_ascii_lowercase()
            )),
            "{text}"
        );
        assert!(
            lower.contains("content-type: multipart/form-data;"),
            "{text}"
        );
        let response = format!(
            "HTTP/1.1 {status} test\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream.write_all(response.as_bytes()).unwrap();
        request
    });
    (format!("http://{addr}/legacy/"), handle)
}

fn read_http_request(stream: &mut TcpStream) -> Vec<u8> {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut out = Vec::new();
    let mut buf = [0u8; 4096];
    let mut content_len = None;
    loop {
        let n = stream.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        out.extend_from_slice(&buf[..n]);
        if content_len.is_none() {
            if let Some(header_end) = find_header_end(&out) {
                let headers = String::from_utf8_lossy(&out[..header_end]);
                content_len = headers.lines().find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    if name.eq_ignore_ascii_case("content-length") {
                        value.trim().parse::<usize>().ok()
                    } else {
                        None
                    }
                });
            }
        }
        if let (Some(header_end), Some(len)) = (find_header_end(&out), content_len) {
            if out.len() >= header_end + 4 + len {
                break;
            }
        }
    }
    out
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    bytes.windows(4).position(|w| w == b"\r\n\r\n")
}

#[test]
fn package_build_emits_deterministic_wheel_and_sdist() {
    let tmp = tempfile::tempdir().unwrap();
    write_project(tmp.path());
    let dist = tmp.path().join("dist");

    let out = run(
        tmp.path(),
        &["package", "build", "--out-dir", dist.to_str().unwrap()],
    );
    assert!(
        out.status.success(),
        "build stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let wheel = dist.join("demo_pkg-0.1.0-py3-none-any.whl");
    let sdist = dist.join("demo-pkg-0.1.0.tar.gz");
    assert!(wheel.exists(), "missing {}", wheel.display());
    assert!(sdist.exists(), "missing {}", sdist.display());

    let wheel_bytes = std::fs::read(&wheel).unwrap();
    let sdist_bytes = std::fs::read(&sdist).unwrap();

    let metadata = read_wheel_entry(&wheel, "demo_pkg-0.1.0.dist-info/METADATA");
    assert!(metadata.contains("Name: demo-pkg"), "{metadata}");
    assert!(metadata.contains("Version: 0.1.0"), "{metadata}");
    assert!(
        metadata.contains("Requires-Dist: requests>=2"),
        "{metadata}"
    );
    assert!(metadata.contains("Provides-Extra: dev"), "{metadata}");
    assert!(
        metadata.contains("Requires-Dist: pytest>=8; extra == \"dev\""),
        "{metadata}"
    );
    let init = read_wheel_entry(&wheel, "demo_pkg/__init__.py");
    assert!(init.contains("__version__"), "{init}");

    let pkg_info = read_sdist_entry(&sdist, "/PKG-INFO");
    assert!(pkg_info.contains("Name: demo-pkg"), "{pkg_info}");
    let sdist_init = read_sdist_entry(&sdist, "/src/demo_pkg/__init__.py");
    assert!(sdist_init.contains("__version__"), "{sdist_init}");

    let replay = run(
        tmp.path(),
        &["package", "build", "--out-dir", dist.to_str().unwrap()],
    );
    assert!(
        replay.status.success(),
        "replay stderr: {}",
        String::from_utf8_lossy(&replay.stderr)
    );
    assert_eq!(
        wheel_bytes,
        std::fs::read(&wheel).unwrap(),
        "wheel replay must be byte-identical"
    );
    assert_eq!(
        sdist_bytes,
        std::fs::read(&sdist).unwrap(),
        "sdist replay must be byte-identical"
    );
}

#[test]
fn publish_dry_run_validates_payloads_without_leaking_token() {
    let tmp = tempfile::tempdir().unwrap();
    write_project(tmp.path());
    let dist = tmp.path().join("dist");
    let build = run(
        tmp.path(),
        &["package", "build", "--out-dir", dist.to_str().unwrap()],
    );
    assert!(build.status.success());
    let wheel = dist.join("demo_pkg-0.1.0-py3-none-any.whl");
    let sdist = dist.join("demo-pkg-0.1.0.tar.gz");
    let pypirc = tmp.path().join(".pypirc");
    std::fs::write(
        &pypirc,
        "[testpypi]\nusername = __token__\npassword = secret-token\n",
    )
    .unwrap();

    let out = run(
        tmp.path(),
        &[
            "publish",
            "--dry-run",
            "--repository",
            "testpypi",
            "--pypirc",
            pypirc.to_str().unwrap(),
            "--json",
            wheel.to_str().unwrap(),
            sdist.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "publish stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"repository_url\": \"https://test.pypi.org/legacy/\""),
        "{stdout}"
    );
    assert!(stdout.contains("\"password_present\": true"), "{stdout}");
    assert!(
        stdout.contains("demo_pkg-0.1.0-py3-none-any.whl"),
        "{stdout}"
    );
    assert!(stdout.contains("demo-pkg-0.1.0.tar.gz"), "{stdout}");
    assert!(
        !stdout.contains("secret-token"),
        "dry-run must not leak token: {stdout}"
    );

    let package_publish = run(
        tmp.path(),
        &[
            "package",
            "publish",
            "--dry-run",
            "--repository",
            "testpypi",
            "--pypirc",
            pypirc.to_str().unwrap(),
            "--json",
        ],
    );
    assert!(
        package_publish.status.success(),
        "package publish stderr: {}",
        String::from_utf8_lossy(&package_publish.stderr)
    );
}

#[test]
fn publish_posts_artifact_without_leaking_token() {
    use mamba::pkgmanage::pkgmgr::auth_header::basic_auth;

    let tmp = tempfile::tempdir().unwrap();
    write_project(tmp.path());
    let dist = tmp.path().join("dist");
    let build = run(
        tmp.path(),
        &["package", "build", "--out-dir", dist.to_str().unwrap()],
    );
    assert!(build.status.success());
    let wheel = dist.join("demo_pkg-0.1.0-py3-none-any.whl");
    let expected_auth = basic_auth("__token__", "secret-token").unwrap();
    let (url, handle) = spawn_publish_server(expected_auth, 200, "ok");

    let out = run(
        tmp.path(),
        &[
            "publish",
            "--publish-url",
            &url,
            "--username",
            "__token__",
            "--password",
            "secret-token",
            "--json",
            wheel.to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "publish stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"mode\": \"upload\""), "{stdout}");
    assert!(stdout.contains("\"response_status\": 200"), "{stdout}");
    assert!(
        stdout.contains("demo_pkg-0.1.0-py3-none-any.whl"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("secret-token"),
        "publish stdout leaked token: {stdout}"
    );
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("secret-token"),
        "publish stderr leaked token: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let request = handle.join().unwrap();
    let request_text = String::from_utf8_lossy(&request);
    assert!(request_text.contains("name=\":action\""), "{request_text}");
    assert!(request_text.contains("file_upload"), "{request_text}");
    assert!(
        request_text.contains("demo_pkg-0.1.0-py3-none-any.whl"),
        "{request_text}"
    );
}

#[test]
fn publish_failure_reports_status_without_leaking_response_body() {
    use mamba::pkgmanage::pkgmgr::auth_header::basic_auth;

    let tmp = tempfile::tempdir().unwrap();
    write_project(tmp.path());
    let dist = tmp.path().join("dist");
    let build = run(
        tmp.path(),
        &["package", "build", "--out-dir", dist.to_str().unwrap()],
    );
    assert!(build.status.success());
    let wheel = dist.join("demo_pkg-0.1.0-py3-none-any.whl");
    let expected_auth = basic_auth("__token__", "secret-token").unwrap();
    let (url, handle) = spawn_publish_server(expected_auth, 403, "denied secret-token");

    let out = run(
        tmp.path(),
        &[
            "publish",
            "--publish-url",
            &url,
            "--username",
            "__token__",
            "--password",
            "secret-token",
            wheel.to_str().unwrap(),
        ],
    );
    assert!(
        !out.status.success(),
        "publish unexpectedly succeeded: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("HTTP 403"), "{stderr}");
    assert!(!stderr.contains("secret-token"), "{stderr}");
    assert!(!stderr.contains("denied"), "{stderr}");
    handle.join().unwrap();
}
