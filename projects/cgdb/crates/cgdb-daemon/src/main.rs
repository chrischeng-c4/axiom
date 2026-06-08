// HANDWRITE-BEGIN gap="missing-generator:hand-written:e10d3641" tracker="2087" reason="Daemon entry — bind UDS + optional TCP, write discovery files, run RPC dispatch loop."
mod handlers;
mod indexer;
mod lens_service;
mod query;

use std::fs;
use std::io::{BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use anyhow::{Context, Result};
use cgdb_core::rpc::{read_message, write_message, RpcRequest};

use handlers::DaemonState;

fn data_root() -> PathBuf {
    dirs::home_dir().expect("home dir").join(".cgdb")
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut port: u16 = 5455;
    let mut tcp_enabled = false;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--tcp" => tcp_enabled = true,
            "--port" => {
                i += 1;
                port = args.get(i).and_then(|s| s.parse().ok()).unwrap_or(5455);
            }
            "--foreground" => {}
            _ => {}
        }
        i += 1;
    }

    let root = data_root();
    fs::create_dir_all(&root)?;
    let sock_path = root.join("daemon.sock");
    let pid_path = root.join("daemon.pid");
    let port_path = root.join("daemon.port");

    if sock_path.exists() {
        let _ = fs::remove_file(&sock_path);
    }
    fs::write(&pid_path, std::process::id().to_string())?;

    let uds = UnixListener::bind(&sock_path)
        .with_context(|| format!("bind UDS {}", sock_path.display()))?;

    let tcp_listen_addr = if tcp_enabled {
        let addr = format!("127.0.0.1:{}", port);
        let tcp = TcpListener::bind(&addr)
            .with_context(|| format!("bind TCP {}", addr))?;
        fs::write(&port_path, port.to_string())?;
        Some((addr, tcp))
    } else {
        let _ = fs::remove_file(&port_path);
        None
    };

    let state = Arc::new(Mutex::new(DaemonState {
        data_root: root.clone(),
        uds_socket: sock_path.to_string_lossy().into_owned(),
        tcp_listen: tcp_listen_addr.as_ref().map(|(a, _)| a.clone()),
        start: Instant::now(),
    }));

    eprintln!(
        "cgdb-daemon: listening on UDS {}{}",
        sock_path.display(),
        tcp_listen_addr
            .as_ref()
            .map(|(a, _)| format!(" + TCP {}", a))
            .unwrap_or_default()
    );

    if let Some((_, tcp)) = tcp_listen_addr {
        let state_tcp = Arc::clone(&state);
        thread::spawn(move || {
            for conn in tcp.incoming() {
                match conn {
                    Ok(stream) => {
                        let s = Arc::clone(&state_tcp);
                        thread::spawn(move || {
                            let _ = handle_tcp(stream, s);
                        });
                    }
                    Err(e) => eprintln!("cgdb-daemon: tcp accept error: {}", e),
                }
            }
        });
    }

    for conn in uds.incoming() {
        match conn {
            Ok(stream) => {
                let s = Arc::clone(&state);
                thread::spawn(move || {
                    let _ = handle_uds(stream, s);
                });
            }
            Err(e) => eprintln!("cgdb-daemon: uds accept error: {}", e),
        }
    }
    Ok(())
}

fn handle_uds(stream: UnixStream, state: Arc<Mutex<DaemonState>>) -> Result<()> {
    let read_stream = stream.try_clone()?;
    let mut reader = BufReader::new(read_stream);
    let mut writer = stream;
    serve_loop(&mut reader, &mut writer, state)
}

fn handle_tcp(stream: TcpStream, state: Arc<Mutex<DaemonState>>) -> Result<()> {
    let read_stream = stream.try_clone()?;
    let mut reader = BufReader::new(read_stream);
    let mut writer = stream;
    serve_loop(&mut reader, &mut writer, state)
}

fn serve_loop<R: std::io::BufRead, W: Write>(
    reader: &mut R,
    writer: &mut W,
    state: Arc<Mutex<DaemonState>>,
) -> Result<()> {
    loop {
        let raw = match read_message(reader)? {
            Some(b) => b,
            None => return Ok(()),
        };
        let req: RpcRequest = serde_json::from_slice(&raw).unwrap_or(RpcRequest {
            jsonrpc: "2.0".into(),
            id: serde_json::Value::Null,
            method: String::new(),
            params: serde_json::Value::Null,
        });
        let resp = handlers::dispatch(&state, req);
        let body = serde_json::to_vec(&resp)?;
        write_message(writer, &body)?;
    }
}

// HANDWRITE-END
