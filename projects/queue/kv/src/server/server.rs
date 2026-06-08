//! TCP server implementation

use super::protocol::{
    encode_blocking_pop_response, encode_mget_response, encode_scan_response, encode_value,
    parse_blocking_pop_payload, parse_cas_payload, parse_incr_payload, parse_key,
    parse_list_push_payload, parse_lock_payload, parse_mget_payload, parse_mset_payload,
    parse_scan_payload, parse_set_payload, read_request, write_response, Command, ProtocolError,
    Status,
};
use super::waiter::WaiterManager;
use crate::{KvEngine, KvKey, KvValue};
use futures::future::select_all;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};

/// KV Server
pub struct KvServer {
    engine: Arc<KvEngine>,
    waiter_manager: Arc<WaiterManager>,
}

impl KvServer {
    /// Create a KV server with an existing engine (for persistence support)
    pub fn with_engine(engine: Arc<KvEngine>) -> Self {
        Self {
            engine,
            waiter_manager: Arc::new(WaiterManager::new()),
        }
    }

    /// Run the server
    pub async fn run(&self, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        info!("Server listening on {}", addr);

        loop {
            let (socket, peer_addr) = listener.accept().await?;
            let engine = self.engine.clone();
            let waiter_manager = self.waiter_manager.clone();

            tokio::spawn(async move {
                debug!("New connection from {}", peer_addr);
                if let Err(e) = handle_connection(socket, engine, waiter_manager).await {
                    warn!("Connection error from {}: {}", peer_addr, e);
                }
                debug!("Connection closed: {}", peer_addr);
            });
        }
    }
}

async fn handle_connection(
    mut socket: TcpStream,
    engine: Arc<KvEngine>,
    waiter_manager: Arc<WaiterManager>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Disable Nagle's algorithm for lower latency
    socket.set_nodelay(true)?;

    let mut buf = vec![0u8; 64 * 1024]; // 64KB buffer

    loop {
        // Read header (5 bytes: 1 cmd + 4 len)
        let n = socket.read(&mut buf[..5]).await?;
        if n == 0 {
            return Ok(()); // Connection closed
        }
        if n < 5 {
            // Partial read, try to read more
            socket.read_exact(&mut buf[n..5]).await?;
        }

        // Parse payload length
        let payload_len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;

        // Read payload
        if payload_len > 0 {
            if buf.len() < 5 + payload_len {
                buf.resize(5 + payload_len, 0);
            }
            socket.read_exact(&mut buf[5..5 + payload_len]).await?;
        }

        // Process request
        let response = match process_request(&buf[..5 + payload_len], &engine, &waiter_manager)
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                let msg = e.to_string();
                write_response(Status::Error, msg.as_bytes())
            }
        };

        // Send response
        socket.write_all(&response).await?;
    }
}

async fn process_request(
    data: &[u8],
    engine: &KvEngine,
    waiter_manager: &WaiterManager,
) -> Result<Vec<u8>, ProtocolError> {
    let (cmd, payload) = read_request(data)?;

    match cmd {
        Command::Ping => Ok(write_response(Status::Ok, b"PONG")),
        Command::Get => {
            let key_str = parse_key(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;

            match engine.get(&key) {
                Some(value) => {
                    let encoded = encode_value(&value);
                    Ok(write_response(Status::Ok, &encoded))
                }
                None => Ok(write_response(Status::Null, &[])),
            }
        }
        Command::Set => {
            let (key_str, ttl_ms, value) = parse_set_payload(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;
            let ttl = ttl_ms.map(Duration::from_millis);

            match engine.set(&key, value, ttl) {
                Ok(()) => Ok(write_response(Status::Ok, &[])),
                Err(e) => Ok(write_response(Status::Error, e.to_string().as_bytes())),
            }
        }
        Command::Del => {
            let key_str = parse_key(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;

            let deleted = engine.delete(&key);
            let result = if deleted { 1u8 } else { 0u8 };
            Ok(write_response(Status::Ok, &[result]))
        }
        Command::Exists => {
            let key_str = parse_key(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;

            let exists = engine.exists(&key);
            let result = if exists { 1u8 } else { 0u8 };
            Ok(write_response(Status::Ok, &[result]))
        }
        Command::Incr => {
            let (key_str, delta) = parse_incr_payload(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;

            match engine.incr(&key, delta) {
                Ok(new_val) => {
                    Ok(write_response(Status::Ok, &new_val.to_be_bytes()))
                }
                Err(e) => {
                    Ok(write_response(Status::Error, e.to_string().as_bytes()))
                }
            }
        }
        Command::Decr => {
            let (key_str, delta) = parse_incr_payload(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;

            match engine.decr(&key, delta) {
                Ok(new_val) => {
                    Ok(write_response(Status::Ok, &new_val.to_be_bytes()))
                }
                Err(e) => {
                    Ok(write_response(Status::Error, e.to_string().as_bytes()))
                }
            }
        }
        Command::Cas => {
            let (key_str, ttl_ms, expected, new_value) = parse_cas_payload(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;
            let ttl = ttl_ms.map(Duration::from_millis);

            match engine.cas(&key, &expected, new_value, ttl) {
                Ok(success) => {
                    let result = if success { 1u8 } else { 0u8 };
                    Ok(write_response(Status::Ok, &[result]))
                }
                Err(e) => {
                    Ok(write_response(Status::Error, e.to_string().as_bytes()))
                }
            }
        }
        Command::Setnx => {
            let (key_str, ttl_ms, value) = parse_set_payload(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;
            let ttl = ttl_ms.map(Duration::from_millis);

            match engine.setnx(&key, value, ttl) {
                Ok(success) => {
                    let result = if success { 1u8 } else { 0u8 };
                    Ok(write_response(Status::Ok, &[result]))
                }
                Err(e) => Ok(write_response(Status::Error, e.to_string().as_bytes())),
            }
        }
        Command::Lock => {
            let (key_str, owner, ttl_ms) = parse_lock_payload(&payload, true)?;
            let key = KvKey::new(&key_str).map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;
            let ttl = Duration::from_millis(ttl_ms.unwrap_or(30000));

            let success = engine.lock(&key, &owner, ttl);
            let result = if success { 1u8 } else { 0u8 };
            Ok(write_response(Status::Ok, &[result]))
        }
        Command::Unlock => {
            let (key_str, owner, _) = parse_lock_payload(&payload, false)?;
            let key = KvKey::new(&key_str).map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;

            match engine.unlock(&key, &owner) {
                Ok(success) => {
                    let result = if success { 1u8 } else { 0u8 };
                    Ok(write_response(Status::Ok, &[result]))
                }
                Err(e) => {
                    Ok(write_response(Status::Error, e.to_string().as_bytes()))
                }
            }
        }
        Command::ExtendLock => {
            let (key_str, owner, ttl_ms) = parse_lock_payload(&payload, true)?;
            let key = KvKey::new(&key_str).map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;
            let ttl = Duration::from_millis(ttl_ms.unwrap_or(30000));

            match engine.extend_lock(&key, &owner, ttl) {
                Ok(success) => {
                    let result = if success { 1u8 } else { 0u8 };
                    Ok(write_response(Status::Ok, &[result]))
                }
                Err(e) => {
                    Ok(write_response(Status::Error, e.to_string().as_bytes()))
                }
            }
        }
        Command::MGet => {
            let keys = parse_mget_payload(&payload)?;
            let kv_keys: Result<Vec<_>, _> = keys.iter()
                .map(|k| KvKey::new(k))
                .collect();

            let kv_keys = kv_keys.map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;

            let key_refs: Vec<&KvKey> = kv_keys.iter().collect();
            let values = engine.mget(&key_refs);

            let encoded = encode_mget_response(&values);
            Ok(write_response(Status::Ok, &encoded))
        }
        Command::MSet => {
            let (pairs, ttl_ms) = parse_mset_payload(&payload)?;
            let ttl = ttl_ms.map(Duration::from_millis);

            let kv_pairs: Result<Vec<_>, _> = pairs.iter()
                .map(|(k, v)| KvKey::new(k).map(|key| (key, v.clone())))
                .collect();

            let kv_pairs = kv_pairs.map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;

            let pair_refs: Vec<(&KvKey, KvValue)> = kv_pairs.iter()
                .map(|(k, v)| (k, v.clone()))
                .collect();

            match engine.mset(&pair_refs, ttl) {
                Ok(()) => Ok(write_response(Status::Ok, &[])),
                Err(e) => Ok(write_response(Status::Error, e.to_string().as_bytes())),
            }
        }
        Command::MDel => {
            let keys = parse_mget_payload(&payload)?; // Same format as MGET
            let kv_keys: Result<Vec<_>, _> = keys.iter()
                .map(|k| KvKey::new(k))
                .collect();

            let kv_keys = kv_keys.map_err(|e| ProtocolError::Io(
                std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string())
            ))?;

            let key_refs: Vec<&KvKey> = kv_keys.iter().collect();
            let deleted = engine.mdel(&key_refs);

            // Return count as u32 big-endian
            Ok(write_response(Status::Ok, &(deleted as u32).to_be_bytes()))
        }
        Command::Info => {
            let info = format!(
                r#"{{"shards":{},"entries":{}}}"#,
                engine.num_shards(),
                engine.len()
            );
            Ok(write_response(Status::Ok, info.as_bytes()))
        }
        Command::Scan => {
            let (prefix, limit) = parse_scan_payload(&payload)?;
            let keys = engine.scan(prefix.as_deref(), limit);
            let encoded = encode_scan_response(&keys);
            Ok(write_response(Status::Ok, &encoded))
        }
        // List operations
        Command::LPush => {
            let (key_str, value) = parse_list_push_payload(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| {
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e.to_string(),
                ))
            })?;

            match engine.lpush(&key, vec![value]) {
                Ok(len) => {
                    // Notify first waiter (removes it from queue)
                    let _ = waiter_manager.notify_one(&key_str);
                    Ok(write_response(Status::Ok, &(len as u32).to_be_bytes()))
                }
                Err(e) => Ok(write_response(Status::Error, e.to_string().as_bytes())),
            }
        }
        Command::RPush => {
            let (key_str, value) = parse_list_push_payload(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| {
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e.to_string(),
                ))
            })?;

            match engine.rpush(&key, vec![value]) {
                Ok(len) => {
                    // Notify first waiter (removes it from queue)
                    let _ = waiter_manager.notify_one(&key_str);
                    Ok(write_response(Status::Ok, &(len as u32).to_be_bytes()))
                }
                Err(e) => Ok(write_response(Status::Error, e.to_string().as_bytes())),
            }
        }
        Command::LPop => {
            let key_str = parse_key(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| {
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e.to_string(),
                ))
            })?;

            match engine.lpop(&key) {
                Some(value) => {
                    let encoded = encode_value(&value);
                    Ok(write_response(Status::Ok, &encoded))
                }
                None => Ok(write_response(Status::Null, &[])),
            }
        }
        Command::RPop => {
            let key_str = parse_key(&payload)?;
            let key = KvKey::new(&key_str).map_err(|e| {
                ProtocolError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e.to_string(),
                ))
            })?;

            match engine.rpop(&key) {
                Some(value) => {
                    let encoded = encode_value(&value);
                    Ok(write_response(Status::Ok, &encoded))
                }
                None => Ok(write_response(Status::Null, &[])),
            }
        }
        Command::BLPop => {
            blocking_pop_handler(engine, waiter_manager, &payload, true).await
        }
        Command::BRPop => {
            blocking_pop_handler(engine, waiter_manager, &payload, false).await
        }
    }
}

/// Try to pop from a key (helper for blocking pop)
fn try_pop_from_key(
    engine: &KvEngine,
    key_str: &str,
    from_left: bool,
) -> Option<(String, KvValue)> {
    let key = KvKey::new(key_str).ok()?;
    let value = if from_left {
        engine.lpop(&key)
    } else {
        engine.rpop(&key)
    }?;
    Some((key_str.to_string(), value))
}

/// Helper for BLPOP/BRPOP - handles blocking wait with proper select! and timeout
async fn blocking_pop_handler(
    engine: &KvEngine,
    waiter_manager: &WaiterManager,
    payload: &[u8],
    from_left: bool,
) -> Result<Vec<u8>, ProtocolError> {
    let (timeout_ms, keys) = parse_blocking_pop_payload(payload)?;

    // First check if any list has elements (in key order)
    for key_str in &keys {
        if let Some((k, v)) = try_pop_from_key(engine, key_str, from_left) {
            let encoded = encode_blocking_pop_response(&k, &v);
            return Ok(write_response(Status::Ok, &encoded));
        }
    }

    // No data available, need to wait
    // Register waiters for all keys
    let mut notify_handles: Vec<(String, Arc<tokio::sync::Notify>)> = Vec::new();
    for key_str in &keys {
        let notify = waiter_manager.register(key_str);
        notify_handles.push((key_str.clone(), notify));
    }

    // Wait loop with proper cleanup on exit
    let wait_result = async {
        loop {
            // Use select_all to wait on ANY notification concurrently
            let futures: Vec<_> = notify_handles
                .iter()
                .map(|(_, notify)| Box::pin(notify.notified()))
                .collect();

            if futures.is_empty() {
                return None;
            }

            // Wait for any one to complete
            let (_, index, _) = select_all(futures).await;

            // After any notification, check ALL keys in order (not just the notified one)
            // This handles spurious wakeups and ensures we get the first available
            for key_str in &keys {
                if let Some((k, v)) = try_pop_from_key(engine, key_str, from_left) {
                    return Some((k, v));
                }
            }

            // Re-register the waiter that was consumed (notify removes it)
            let (key_str, _) = &notify_handles[index];
            let new_notify = waiter_manager.register(key_str);
            notify_handles[index].1 = new_notify;
        }
    };

    // Apply timeout if specified (0 = block indefinitely)
    let result = if timeout_ms == 0 {
        wait_result.await
    } else {
        let timeout = Duration::from_millis(timeout_ms);
        match tokio::time::timeout(timeout, wait_result).await {
            Ok(r) => r,
            Err(_) => None, // Timeout
        }
    };

    // Cleanup: unregister all remaining waiters
    for (key_str, notify) in &notify_handles {
        waiter_manager.unregister(key_str, notify);
    }

    match result {
        Some((key_str, value)) => {
            let encoded = encode_blocking_pop_response(&key_str, &value);
            Ok(write_response(Status::Ok, &encoded))
        }
        None => Ok(write_response(Status::Null, &[])),
    }
}
