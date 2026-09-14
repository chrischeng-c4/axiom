//! One ordered state-machine worker. Only selection and completion touch the
//! Raft-node mutex; file mapping, capacity waits, apply, and snapshot encoding
//! run on the blocking pool under the separate state-machine serial boundary.

use super::*;
use raft_core::EntryKind;

impl Shared {
    /// Called while the node mutex is held, after its current state is durable.
    pub(super) fn schedule_apply(self: &Arc<Self>, node: &RaftNode) {
        if self.apply_stopped.load(Ordering::Acquire)
            || self.latched_failure.lock().unwrap().is_some()
            || node.peek_next_committed_identity().is_none()
            || self
                .apply_running
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
        {
            return;
        }
        self.apply_tracker.active.fetch_add(1, Ordering::SeqCst);
        let tracker = Arc::clone(&self.apply_tracker);
        let shared = Arc::clone(self);
        tokio::task::spawn_blocking(move || {
            let _active = RpcGuard { tracker };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                shared.run_ordered_apply()
            }));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) => shared.fail_ordered_apply("state-machine-apply", &error),
                Err(_) => shared.fail_ordered_apply(
                    "state-machine-apply-panic",
                    &anyhow!("state-machine worker panicked"),
                ),
            }
        });
    }

    fn fail_ordered_apply(&self, operation: &'static str, error: &anyhow::Error) {
        // Reset while holding node, just as normal exhaustion does. A new
        // scheduler cannot mistake a still-running worker for an idle one.
        let _node = self.node.blocking_lock();
        tracing::error!(%error, operation, "raft: committed apply failed; retain the head until restart");
        let mut failure = self.latched_failure.lock().unwrap();
        if failure.is_none() {
            *failure = Some(StorageFailed {
                node_id: self.id,
                operation,
                path: self.store.path().to_path_buf(),
                kind: std::io::ErrorKind::InvalidData,
            });
        }
        self.apply_running.store(false, Ordering::Release);
    }

    fn run_ordered_apply(&self) -> Result<()> {
        loop {
            // Restore and snapshot encoding share this lease, never node. A
            // lease is released between records so they can take a clean cut.
            let _state_machine = self.snapshot_install.blocking_lock();
            let selected = {
                let mut node = self.node.blocking_lock();
                if self.latched_failure.lock().unwrap().is_some() {
                    self.apply_running.store(false, Ordering::Release);
                    return Ok(());
                }
                let Some((index, term, kind)) = node.peek_next_committed_identity() else {
                    self.apply_running.store(false, Ordering::Release);
                    return Ok(());
                };
                if kind == EntryKind::Config {
                    if !node.finish_committed_identity(index, term) {
                        anyhow::bail!("committed configuration identity changed at {index}");
                    }
                    self.persist(&node)?;
                    None
                } else {
                    let source = self.store.pin_committed_command(index, term)?;
                    let permit = {
                        let mut pending = self
                            .pending_admission
                            .lock()
                            .unwrap_or_else(|p| p.into_inner());
                        pending.retain(|(at, entry_term), _| *at != index || *entry_term == term);
                        pending.remove(&(index, term))
                    };
                    Some((index, term, source, permit))
                }
            };
            if let Some((index, term, source, permit)) = selected {
                if self.sm.applied_index() < index {
                    let command = source.map()?;
                    self.sm.apply_admitted(index, command.command(), permit)?;
                    if self.sm.applied_index() < index {
                        anyhow::bail!(
                            "state machine returned success without applying index {index}"
                        );
                    }
                }
                let mut node = self.node.blocking_lock();
                if !node.finish_committed_identity(index, term) {
                    anyhow::bail!("committed command identity changed after applying {index}");
                }
                self.persist(&node)?;
                self.applied_tx.send_replace(self.sm.applied_index());
            }
            self.capture_periodic_snapshot()?;
        }
    }

    fn capture_periodic_snapshot(&self) -> Result<()> {
        let SnapshotPolicy::EveryEntries(every) = self.cfg.snapshot else {
            return Ok(());
        };
        let applied = self.sm.applied_index();
        {
            let node = self.node.blocking_lock();
            if applied == 0 || applied.saturating_sub(node.snapshot_index()) < every {
                return Ok(());
            }
        }
        let mut sink = ChunkSink::new(SNAPSHOT_CHUNK_SIZE);
        if let Err(error) = self.sm.snapshot_at(applied, &mut sink) {
            tracing::warn!(%error, "raft: snapshot capture failed; skip compaction");
            return Ok(());
        }
        let mut node = self.node.blocking_lock();
        node.compact(applied, sink.into_bytes());
        self.persist(&node)?;
        Ok(())
    }
}

impl Shared {
    /// The caller owns the state-machine serial lease on the blocking pool.
    pub(super) fn install_snapshot_serial(
        self: &Arc<Self>,
        from: NodeId,
        req: InstallSnapshotReq,
    ) -> InstallSnapshotResp {
        let validation_required = {
            let node = self.node.blocking_lock();
            req.term >= node.current_term() && req.snapshot_index > node.snapshot_index()
        };
        if validation_required {
            if let Err(error) = self
                .sm
                .validate_snapshot(&mut std::io::Cursor::new(req.data.as_slice()))
            {
                tracing::warn!(%error, "raft: state machine rejected incoming snapshot");
                let mut node = self.node.blocking_lock();
                node.reject_install_snapshot(req);
                if self.persist(&node).is_err() {
                    return InstallSnapshotResp {
                        term: 0,
                        accepted: false,
                        snapshot_index: 0,
                    };
                }
                return match take_reply(&mut node, from) {
                    Some(RaftMsg::InstallSnapshotResp(response)) => response,
                    _ => InstallSnapshotResp {
                        term: node.current_term(),
                        accepted: false,
                        snapshot_index: node.snapshot_index(),
                    },
                };
            }
        }
        let (restore, bytes, response) = {
            let mut node = self.node.blocking_lock();
            let restore =
                req.term >= node.current_term() && req.snapshot_index > node.snapshot_index();
            node.handle(from, RaftMsg::InstallSnapshot(req));
            if self.persist(&node).is_err() {
                let _ = take_reply(&mut node, from);
                return InstallSnapshotResp {
                    term: 0,
                    accepted: false,
                    snapshot_index: 0,
                };
            }
            let bytes = if restore {
                node.take_installed_snapshot()
            } else {
                None
            };
            let response = match take_reply(&mut node, from) {
                Some(RaftMsg::InstallSnapshotResp(response)) => response,
                _ => InstallSnapshotResp {
                    term: node.current_term(),
                    accepted: false,
                    snapshot_index: node.snapshot_index(),
                },
            };
            (restore, bytes, response)
        };
        if restore {
            let result = bytes
                .ok_or_else(|| anyhow!("durable snapshot has no restore bytes"))
                .and_then(|bytes| self.sm.restore(&mut std::io::Cursor::new(bytes)));
            if let Err(error) = result {
                tracing::error!(%error, "raft: durable snapshot restore failed; latch node until restart");
                *self.latched_failure.lock().unwrap() = Some(StorageFailed {
                    node_id: self.id,
                    operation: "state-machine-restore",
                    path: self.store.path().to_path_buf(),
                    kind: std::io::ErrorKind::InvalidData,
                });
                return InstallSnapshotResp {
                    accepted: false,
                    ..response
                };
            }
            self.pending_admission
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .retain(|(index, _), _| *index > response.snapshot_index);
            self.applied_tx.send_replace(self.sm.applied_index());
        }
        let mut node = self.node.blocking_lock();
        self.apply_ready(&mut node);
        response
    }
}
