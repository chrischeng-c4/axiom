//! Shared construction for one replicated service host.
//!
//! The runtime owns the startup order. A service supplies only its membership
//! policy, state machine, storage location, and service-specific names.

use std::path::Path;
use std::sync::Arc;

use anyhow::{bail, Context, Result};

use crate::{
    ClusterTopology, FsyncPolicy, HostConfig, PeerTransport, RaftHost, RaftStateMachine, RaftStore,
};

/// Product policy applied after the shared topology has been read and checked.
pub trait MembershipPolicy: Send + Sync + 'static {
    fn validate(&self, topology: &ClusterTopology) -> Result<()>;
}

/// A fully started replicated host and the transport used by its peer server.
pub struct ReplicaHostRuntime {
    pub host: Arc<RaftHost>,
    pub peer_transport: PeerTransport,
    pub peer_port: u16,
    pub topology: ClusterTopology,
}

/// Builds the common topology, peer transport, durable store, and Raft host.
pub struct ReplicaHostBuilder<P> {
    service_prefix: String,
    headless_service: String,
    peer_port: u16,
    peers_override: String,
    scheme: String,
    membership_policy: P,
}

impl<P> ReplicaHostBuilder<P>
where
    P: MembershipPolicy,
{
    pub fn new(
        service_prefix: impl Into<String>,
        headless_service: impl Into<String>,
        peer_port: u16,
        peers_override: impl Into<String>,
        scheme: impl Into<String>,
        membership_policy: P,
    ) -> Result<Self> {
        let service_prefix = service_prefix.into();
        let headless_service = headless_service.into();
        let peers_override = peers_override.into();
        let scheme = scheme.into();
        if service_prefix.trim().is_empty() {
            bail!("raft service prefix must not be empty");
        }
        if headless_service.trim().is_empty() {
            bail!("raft headless service must not be empty");
        }
        if peers_override.trim().is_empty() {
            bail!("raft peer override environment variable must not be empty");
        }
        if peer_port == 0 {
            bail!("raft peer port must be greater than zero");
        }
        if !matches!(scheme.as_str(), "http" | "https") {
            bail!("raft peer URL scheme must be http or https");
        }
        Ok(Self {
            service_prefix,
            headless_service,
            peer_port,
            peers_override,
            scheme,
            membership_policy,
        })
    }

    /// Read the standard StatefulSet topology and apply product policy.
    pub fn topology(&self) -> Result<ClusterTopology> {
        let topology = ClusterTopology::from_env_with_scheme(
            &self.service_prefix,
            &self.headless_service,
            self.peer_port,
            &self.peers_override,
            &self.scheme,
        )?;
        self.membership_policy.validate(&topology)?;
        Ok(topology)
    }

    /// Start a host whose peer traffic is protected by required mutual TLS.
    ///
    /// The product chooses this secure startup path. The runtime then owns the
    /// exact order and never falls back to a clear-text peer transport.
    pub fn build_secure<S>(
        &self,
        data_dir: &Path,
        state_machine: Arc<S>,
        peer_tls_env_prefix: &str,
        fsync_policy: FsyncPolicy,
        host_config: HostConfig,
    ) -> Result<ReplicaHostRuntime>
    where
        S: RaftStateMachine,
    {
        if self.scheme != "https" {
            bail!("secure replica host requires the https peer URL scheme");
        }
        let tls = peer_tls::PeerTlsConfig::from_env(peer_tls_env_prefix)
            .context("load replicated peer mTLS material")?
            .context("replicated peer mTLS material is required")?;
        if !tls.required {
            bail!("replicated peer mTLS must be required; set {peer_tls_env_prefix}_MTLS=on");
        }
        let peer_transport =
            PeerTransport::from_config(&tls).context("build replicated peer mTLS transport")?;
        let topology = self.topology()?;
        let store_dir = data_dir
            .to_str()
            .context("raft data directory must be valid UTF-8")?;
        let store = RaftStore::open(store_dir, topology.node_id, fsync_policy)
            .context("open durable raft store")?;
        let host = Arc::new(RaftHost::spawn_with_peer_transport(
            topology.node_id,
            topology.membership.clone(),
            topology.peers.clone(),
            store,
            state_machine as Arc<dyn RaftStateMachine>,
            host_config,
            peer_transport.clone(),
        ));
        Ok(ReplicaHostRuntime {
            host,
            peer_transport,
            peer_port: self.peer_port,
            topology,
        })
    }
}
