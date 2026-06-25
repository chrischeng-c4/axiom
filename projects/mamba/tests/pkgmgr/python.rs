//! CLI integration tests for `mamba python`.

use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run_in(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .current_dir(dir)
        .args(args)
        .output()
        .expect("spawn mamba")
}

fn run_with_data_dir(data: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .args(args)
        .env("UV_DATA_DIR", data)
        .output()
        .expect("spawn mamba")
}

fn write_fake_python(dir: &Path, version: &str) -> PathBuf {
    let path = dir.join("python");
    let body = format!(
        r#"#!/bin/sh
if [ "$1" = "-I" ]; then
  echo "{}"
  exit 0
fi
echo "fake-python $@"
"#,
        version.replace('.', " ")
    );
    std::fs::write(&path, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&path).unwrap().permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(&path, perm).unwrap();
    }
    path
}

fn fake_standalone_archive(version: &str) -> Vec<u8> {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut tar_bytes);
        let body = format!(
            r#"#!/bin/sh
if [ "$1" = "-I" ]; then
  echo "{}"
  exit 0
fi
echo "downloaded-python $@"
"#,
            version.replace('.', " ")
        );
        let mut header = tar::Header::new_gnu();
        header.set_path("python/install/bin/python3").unwrap();
        header.set_size(body.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        builder
            .append(&header, body.as_bytes())
            .expect("append fake python");
        builder.finish().unwrap();
    }
    zstd::stream::encode_all(tar_bytes.as_slice(), 0).unwrap()
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn spawn_archive_server(bytes: Vec<u8>) -> (String, JoinHandle<Vec<u8>>) {
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
                        panic!("timed out waiting for Python archive download");
                    }
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(e) => panic!("accept Python archive download: {e}"),
            }
        };
        stream.set_nonblocking(false).unwrap();
        let request = read_http_headers(&mut stream);
        let text = String::from_utf8_lossy(&request);
        assert!(text.starts_with("GET /cpython.tar.zst HTTP/1.1"), "{text}");
        let response = format!(
            "HTTP/1.1 200 ok\r\ncontent-type: application/zstd\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
            bytes.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        stream.write_all(&bytes).unwrap();
        request
    });
    (format!("http://{addr}/cpython.tar.zst"), handle)
}

fn read_http_headers(stream: &mut TcpStream) -> Vec<u8> {
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    let mut out = Vec::new();
    let mut buf = [0u8; 1024];
    loop {
        let n = stream.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        out.extend_from_slice(&buf[..n]);
        if out.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    out
}

#[test]
fn python_pin_writes_python_version_file() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run_in(tmp.path(), &["python", "pin", "3.12"]);
    assert!(
        out.status.success(),
        "python pin must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let body = std::fs::read_to_string(tmp.path().join(".python-version")).unwrap();
    assert_eq!(body, "3.12\n");
}

#[test]
fn python_pin_rejects_any_request() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run_in(tmp.path(), &["python", "pin", "any"]);
    assert!(!out.status.success(), "pinning any must fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unconstrained"),
        "stderr explains rejection: {stderr:?}"
    );
}

#[test]
fn python_dir_honors_uv_data_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    let out = Command::new(mamba_bin())
        .args(["python", "dir"])
        .env("UV_DATA_DIR", &data)
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "python dir must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        data.join("python").to_string_lossy()
    );
}

#[test]
fn python_dir_bin_honors_uv_data_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    let out = Command::new(mamba_bin())
        .args(["python", "dir", "--bin"])
        .env("UV_DATA_DIR", &data)
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "python dir --bin must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        data.join("python").join("bin").to_string_lossy()
    );
}

#[test]
fn python_list_succeeds_when_path_has_no_interpreters() {
    let tmp = tempfile::tempdir().unwrap();
    let empty_path = tmp.path().join("empty");
    std::fs::create_dir_all(&empty_path).unwrap();
    let out = Command::new(mamba_bin())
        .args(["python", "list"])
        .env("PATH", &empty_path)
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "python list must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
}

#[test]
fn python_find_fails_cleanly_when_no_interpreter_matches() {
    let tmp = tempfile::tempdir().unwrap();
    let empty_path = tmp.path().join("empty");
    std::fs::create_dir_all(&empty_path).unwrap();
    std::fs::write(tmp.path().join(".python-version"), "3.12\n").unwrap();
    let out = Command::new(mamba_bin())
        .current_dir(tmp.path())
        .args(["python", "find"])
        .env("PATH", &empty_path)
        .output()
        .expect("spawn mamba");
    assert!(!out.status.success(), "find must fail without a match");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no installed Python matches 3.12"),
        "stderr names missing request: {stderr:?}"
    );
}

#[test]
fn python_install_download_update_shell_and_uninstall_are_local_first() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    let source_dir = tmp.path().join("source");
    std::fs::create_dir_all(&source_dir).unwrap();
    let fake_python = write_fake_python(&source_dir, "3.12.7");

    let install = run_with_data_dir(
        &data,
        &[
            "python",
            "install",
            "3.12.7",
            "--source",
            fake_python.to_str().unwrap(),
        ],
    );
    assert!(
        install.status.success(),
        "python install must succeed; stderr: {}",
        String::from_utf8_lossy(&install.stderr)
    );

    let root = data.join("python");
    let version_dir = root.join("3.12.7");
    assert!(version_dir.join("bin/python").exists());
    assert!(root.join("bin/python").exists());
    assert!(root.join("bin/python3").exists());
    assert!(root.join("bin/python3.12").exists());
    assert!(root.join("bin/python3.12.7").exists());

    let update_shell = run_with_data_dir(&data, &["python", "update-shell", "--shell", "bash"]);
    assert!(
        update_shell.status.success(),
        "python update-shell must succeed; stderr: {}",
        String::from_utf8_lossy(&update_shell.stderr)
    );
    let shell_stdout = String::from_utf8_lossy(&update_shell.stdout);
    assert!(shell_stdout.contains("# >>> mamba initialize >>>"));
    assert!(shell_stdout.contains(&format!(
        "export PATH=\"{}:$PATH\"",
        root.join("bin").display()
    )));

    let download = run_with_data_dir(
        &data,
        &[
            "python",
            "download",
            "3.12.7",
            "--source",
            fake_python.to_str().unwrap(),
        ],
    );
    assert!(
        download.status.success(),
        "python download must be idempotent; stderr: {}",
        String::from_utf8_lossy(&download.stderr)
    );

    let uninstall = run_with_data_dir(&data, &["python", "uninstall", "3.12.7"]);
    assert!(
        uninstall.status.success(),
        "python uninstall must succeed; stderr: {}",
        String::from_utf8_lossy(&uninstall.stderr)
    );
    assert!(!version_dir.exists());
    assert!(!root.join("bin/python3.12.7").exists());
}

#[test]
fn python_download_fetches_standalone_archive_and_builds_launchers() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    let archive = fake_standalone_archive("3.12.8");
    let expected_sha = sha256_hex(&archive);
    let (url, handle) = spawn_archive_server(archive);

    let download = run_with_data_dir(
        &data,
        &[
            "python",
            "download",
            "3.12.8",
            "--url",
            &url,
            "--sha256",
            &expected_sha,
        ],
    );
    assert!(
        download.status.success(),
        "python download must succeed; stderr: {}",
        String::from_utf8_lossy(&download.stderr)
    );
    handle.join().unwrap();

    let root = data.join("python");
    let version_dir = root.join("3.12.8");
    assert!(version_dir.join("python/install/bin/python3").exists());
    assert!(version_dir.join("bin/python").exists());
    assert!(root.join("bin/python").exists());
    assert!(root.join("bin/python3").exists());
    assert!(root.join("bin/python3.12").exists());
    assert!(root.join("bin/python3.12.8").exists());

    let probe = Command::new(root.join("bin/python3.12.8"))
        .args(["-I", "-c", "import sys; print(sys.version_info[:3])"])
        .output()
        .expect("run downloaded launcher");
    assert!(
        probe.status.success(),
        "downloaded launcher stderr: {}",
        String::from_utf8_lossy(&probe.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "3 12 8");

    let manifest = std::fs::read_to_string(version_dir.join("manifest.toml")).unwrap();
    assert!(manifest.contains("archive_sha256"));
    assert!(manifest.contains(&expected_sha));
}
