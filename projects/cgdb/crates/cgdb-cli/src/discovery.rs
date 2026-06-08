// HANDWRITE-BEGIN gap="missing-generator:hand-written:8b663ba2" tracker="2087" reason="Read ~/.cgdb/daemon.{sock,port} discovery files; honor --socket / --port / CGDB_SOCKET / CGDB_PORT overrides."
use std::path::PathBuf;

pub fn data_root() -> PathBuf {
    dirs::home_dir().expect("home dir").join(".cgdb")
}

pub fn sock_path() -> PathBuf {
    data_root().join("daemon.sock")
}

pub fn pid_path() -> PathBuf {
    data_root().join("daemon.pid")
}

pub fn port_path() -> PathBuf {
    data_root().join("daemon.port")
}

pub fn read_port() -> Option<u16> {
    std::fs::read_to_string(port_path()).ok()?.trim().parse().ok()
}
// HANDWRITE-END
