// HANDWRITE-BEGIN gap="missing-generator:logic:b147179b" tracker="#1704" reason="Prefix-scoped adapter regression coverage without service application environment mutation."
use std::sync::Mutex;

use raft_host::PeerTlsConfig;

const PREFIX: &str = "RAFT_HOST_PEER_TLS_TEST";
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn clear_env() {
    std::env::remove_var(format!("{PREFIX}_TLS_CERT"));
    std::env::remove_var(format!("{PREFIX}_TLS_KEY"));
    std::env::remove_var(format!("{PREFIX}_TLS_CA"));
    std::env::remove_var(format!("{PREFIX}_MTLS"));
}

#[test]
fn prefix_contract_is_all_or_nothing_and_stages_validated_paths() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear_env();
    assert!(PeerTlsConfig::from_env(PREFIX).unwrap().is_none());

    std::env::set_var(format!("{PREFIX}_TLS_CERT"), "/tmp/raft-host-peer-tls-cert");
    let partial = PeerTlsConfig::from_env(PREFIX).unwrap_err();
    assert!(partial.to_string().contains("must all be set together"));
    clear_env();

    let directory = std::env::temp_dir().join(format!("raft-host-peer-tls-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    for name in ["cert.pem", "key.pem", "ca.pem"] {
        std::fs::write(directory.join(name), b"fixture").unwrap();
    }
    std::env::set_var(format!("{PREFIX}_TLS_CERT"), directory.join("cert.pem"));
    std::env::set_var(format!("{PREFIX}_TLS_KEY"), directory.join("key.pem"));
    std::env::set_var(format!("{PREFIX}_TLS_CA"), directory.join("ca.pem"));
    std::env::set_var(format!("{PREFIX}_MTLS"), "on");

    let config = PeerTlsConfig::from_env(PREFIX)
        .unwrap()
        .expect("configured peer TLS");
    assert_eq!(config.cert(), directory.join("cert.pem"));
    assert_eq!(config.key(), directory.join("key.pem"));
    assert_eq!(config.ca(), directory.join("ca.pem"));
    assert!(config.mtls_required());
    assert!(config.rustls_server_config().is_err());
    assert!(config.rustls_client_config().is_err());

    std::fs::remove_dir_all(directory).ok();
    clear_env();
}
// HANDWRITE-END
