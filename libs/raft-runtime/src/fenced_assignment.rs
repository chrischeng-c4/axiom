// HANDWRITE-BEGIN gap="missing-generator:logic:raft-runtime-fenced-assignment" tracker="#1854" reason="Application-neutral committed executor ownership, fencing, expiry, and stale-outcome transition primitive."
//! Deterministic committed executor ownership and fencing.
//!
//! [`FencedAssignment`] is intended to live inside a service-owned
//! [`crate::RaftStateMachine`]. The application still owns the assignment key
//! (message, task, queue, shard, ...), command encoding, and domain outcome;
//! this module only supplies the transition invariants shared by effectful
//! services. Calling these methods outside committed state-machine apply does
//! not make an assignment authoritative.

use std::error::Error;
use std::fmt;

use raft_core::NodeId;
use serde::{Deserialize, Serialize};

/// Monotonic token identifying one committed ownership generation.
pub type AssignmentEpoch = u64;

/// Proof that one replica owns an assignment at one fencing epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FenceToken {
    pub owner: NodeId,
    pub epoch: AssignmentEpoch,
}

/// The currently committed, time-bounded assignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveAssignment {
    pub token: FenceToken,
    /// Absolute timestamp chosen by the proposer before the command enters
    /// Raft. State-machine apply must never read a replica-local clock.
    pub expires_at_ms: u64,
}

/// Durable fencing state for one application-owned assignment key.
///
/// Expiry is deliberately an explicit transition: an elapsed wall clock does
/// not silently clear ownership on one replica. The application proposes an
/// expire/reclaim command containing `now_ms`, applies [`expire`](Self::expire)
/// on every replica, and may assign a new owner only after that commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FencedAssignment {
    epoch: AssignmentEpoch,
    active: Option<ActiveAssignment>,
}

impl FencedAssignment {
    /// Empty ownership state. The first assignment receives epoch 1.
    pub const fn idle() -> Self {
        Self {
            epoch: 0,
            active: None,
        }
    }

    /// Highest epoch ever assigned, retained while idle so epochs never repeat.
    pub const fn epoch(&self) -> AssignmentEpoch {
        self.epoch
    }

    /// The current committed assignment, including one that has passed its
    /// deadline but has not yet been explicitly expired by a committed command.
    pub const fn active(&self) -> Option<ActiveAssignment> {
        self.active
    }

    /// A token is available only after assignment commit. Executor adapters use
    /// this as their commit-before-effect gate.
    pub const fn token(&self) -> Option<FenceToken> {
        match self.active {
            Some(active) => Some(active.token),
            None => None,
        }
    }

    /// Assign an idle key to `owner`, incrementing the fencing epoch.
    pub fn assign(
        &mut self,
        owner: NodeId,
        now_ms: u64,
        expires_at_ms: u64,
    ) -> Result<FenceToken, AssignmentError> {
        if expires_at_ms <= now_ms {
            return Err(AssignmentError::InvalidExpiry {
                now_ms,
                expires_at_ms,
            });
        }
        if let Some(active) = self.active {
            return Err(AssignmentError::AlreadyAssigned(active));
        }
        let epoch = self
            .epoch
            .checked_add(1)
            .ok_or(AssignmentError::EpochExhausted)?;
        let token = FenceToken { owner, epoch };
        self.epoch = epoch;
        self.active = Some(ActiveAssignment {
            token,
            expires_at_ms,
        });
        Ok(token)
    }

    /// Validate that `token` is the current unexpired owner at `now_ms`.
    pub fn validate(
        &self,
        token: FenceToken,
        now_ms: u64,
    ) -> Result<ActiveAssignment, AssignmentError> {
        let active = self.active.ok_or(AssignmentError::Unassigned {
            current_epoch: self.epoch,
        })?;
        if token.epoch != active.token.epoch {
            return Err(AssignmentError::StaleEpoch {
                current: active.token.epoch,
                provided: token.epoch,
            });
        }
        if token.owner != active.token.owner {
            return Err(AssignmentError::OwnerMismatch {
                current: active.token.owner,
                provided: token.owner,
            });
        }
        if now_ms >= active.expires_at_ms {
            return Err(AssignmentError::Expired {
                expires_at_ms: active.expires_at_ms,
                now_ms,
            });
        }
        Ok(active)
    }

    /// Extend an unexpired assignment. Renewals cannot shorten the deadline.
    pub fn renew(
        &mut self,
        token: FenceToken,
        now_ms: u64,
        expires_at_ms: u64,
    ) -> Result<ActiveAssignment, AssignmentError> {
        let active = self.validate(token, now_ms)?;
        if expires_at_ms <= active.expires_at_ms {
            return Err(AssignmentError::ExpiryNotExtended {
                current: active.expires_at_ms,
                proposed: expires_at_ms,
            });
        }
        let renewed = ActiveAssignment {
            token,
            expires_at_ms,
        };
        self.active = Some(renewed);
        Ok(renewed)
    }

    /// Complete, cancel, or voluntarily release an unexpired assignment.
    /// The epoch is retained and the next assignment receives a higher token.
    pub fn release(
        &mut self,
        token: FenceToken,
        now_ms: u64,
    ) -> Result<ActiveAssignment, AssignmentError> {
        let active = self.validate(token, now_ms)?;
        self.active = None;
        Ok(active)
    }

    /// Apply the committed expiry/reclaim transition after the deadline.
    pub fn expire(&mut self, now_ms: u64) -> Result<ActiveAssignment, AssignmentError> {
        let active = self.active.ok_or(AssignmentError::Unassigned {
            current_epoch: self.epoch,
        })?;
        if now_ms < active.expires_at_ms {
            return Err(AssignmentError::NotExpired {
                expires_at_ms: active.expires_at_ms,
                now_ms,
            });
        }
        self.active = None;
        Ok(active)
    }
}

/// Deterministic transition rejection returned identically by every replica.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentError {
    InvalidExpiry {
        now_ms: u64,
        expires_at_ms: u64,
    },
    AlreadyAssigned(ActiveAssignment),
    Unassigned {
        current_epoch: AssignmentEpoch,
    },
    StaleEpoch {
        current: AssignmentEpoch,
        provided: AssignmentEpoch,
    },
    OwnerMismatch {
        current: NodeId,
        provided: NodeId,
    },
    Expired {
        expires_at_ms: u64,
        now_ms: u64,
    },
    NotExpired {
        expires_at_ms: u64,
        now_ms: u64,
    },
    ExpiryNotExtended {
        current: u64,
        proposed: u64,
    },
    EpochExhausted,
}

impl fmt::Display for AssignmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidExpiry {
                now_ms,
                expires_at_ms,
            } => write!(
                f,
                "assignment expiry {expires_at_ms} must be after {now_ms}"
            ),
            Self::AlreadyAssigned(active) => write!(
                f,
                "assignment is owned by node {} at epoch {} until {}",
                active.token.owner, active.token.epoch, active.expires_at_ms
            ),
            Self::Unassigned { current_epoch } => {
                write!(f, "assignment is idle at epoch {current_epoch}")
            }
            Self::StaleEpoch { current, provided } => {
                write!(
                    f,
                    "stale assignment epoch {provided}; current epoch is {current}"
                )
            }
            Self::OwnerMismatch { current, provided } => {
                write!(
                    f,
                    "assignment owner {provided} does not match current owner {current}"
                )
            }
            Self::Expired {
                expires_at_ms,
                now_ms,
            } => write!(
                f,
                "assignment expired at {expires_at_ms}; current time is {now_ms}"
            ),
            Self::NotExpired {
                expires_at_ms,
                now_ms,
            } => write!(
                f,
                "assignment expires at {expires_at_ms}; current time is {now_ms}"
            ),
            Self::ExpiryNotExtended { current, proposed } => write!(
                f,
                "renewal expiry {proposed} must be greater than current expiry {current}"
            ),
            Self::EpochExhausted => f.write_str("assignment epoch exhausted"),
        }
    }
}

impl Error for AssignmentError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_token_exists_before_assignment() {
        let state = FencedAssignment::idle();
        assert_eq!(state.epoch(), 0);
        assert_eq!(state.token(), None);
    }

    #[test]
    fn assignment_is_exclusive_until_explicit_release_or_expiry() {
        let mut state = FencedAssignment::idle();
        let token = state.assign(1, 10, 20).unwrap();
        assert_eq!(token, FenceToken { owner: 1, epoch: 1 });
        assert!(matches!(
            state.assign(2, 21, 30),
            Err(AssignmentError::AlreadyAssigned(_))
        ));
        assert!(matches!(
            state.expire(19),
            Err(AssignmentError::NotExpired { .. })
        ));
        state.expire(20).unwrap();
        let next = state.assign(2, 20, 30).unwrap();
        assert_eq!(next, FenceToken { owner: 2, epoch: 2 });
    }

    #[test]
    fn stale_owner_is_rejected_after_reassignment() {
        let mut state = FencedAssignment::idle();
        let old = state.assign(1, 0, 10).unwrap();
        state.expire(10).unwrap();
        let current = state.assign(2, 10, 20).unwrap();
        assert!(matches!(
            state.validate(old, 11),
            Err(AssignmentError::StaleEpoch {
                current: 2,
                provided: 1
            })
        ));
        assert_eq!(state.validate(current, 11).unwrap().token, current);
    }

    #[test]
    fn renewal_requires_current_owner_epoch_and_later_expiry() {
        let mut state = FencedAssignment::idle();
        let token = state.assign(3, 100, 200).unwrap();
        assert!(matches!(
            state.renew(FenceToken { owner: 4, ..token }, 150, 250),
            Err(AssignmentError::OwnerMismatch { .. })
        ));
        assert!(matches!(
            state.renew(token, 150, 200),
            Err(AssignmentError::ExpiryNotExtended { .. })
        ));
        assert_eq!(state.renew(token, 150, 250).unwrap().expires_at_ms, 250);
    }

    #[test]
    fn release_retains_epoch_and_fences_late_completion() {
        let mut state = FencedAssignment::idle();
        let token = state.assign(5, 0, 100).unwrap();
        state.release(token, 50).unwrap();
        assert_eq!(state.epoch(), 1);
        assert!(matches!(
            state.validate(token, 51),
            Err(AssignmentError::Unassigned { current_epoch: 1 })
        ));
        assert_eq!(state.assign(5, 51, 100).unwrap().epoch, 2);
    }

    #[test]
    fn identical_commands_produce_identical_replica_state() {
        let mut a = FencedAssignment::idle();
        let mut b = FencedAssignment::idle();
        for state in [&mut a, &mut b] {
            let first = state.assign(1, 1_000, 2_000).unwrap();
            state.renew(first, 1_500, 2_500).unwrap();
            state.expire(2_500).unwrap();
            state.assign(2, 2_500, 3_500).unwrap();
        }
        assert_eq!(a, b);
        let bytes = serde_json::to_vec(&a).unwrap();
        assert_eq!(
            serde_json::from_slice::<FencedAssignment>(&bytes).unwrap(),
            a
        );
    }
}
// HANDWRITE-END
