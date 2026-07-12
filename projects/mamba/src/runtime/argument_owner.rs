// HANDWRITE-BEGIN gap="missing-generator:mamba-argument-owner-frame" tracker="#1451" reason="Thread-local frame identity, matching, and nested cleanup require a runtime transaction primitive."
use crate::runtime::value::MbValue;
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

/// Explicit provenance for one physical call argument. `owner` originates in
/// a caller companion slot; this frame deliberately never derives it from
/// `value_bits`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ArgumentOwnerSlot {
    value_bits: u64,
    owner: MbValue,
}

impl ArgumentOwnerSlot {
    pub(crate) fn new(value: MbValue, owner: MbValue) -> Self {
        Self {
            value_bits: value.to_bits(),
            owner,
        }
    }
}

#[derive(Debug)]
struct ArgumentOwnerFrame {
    id: u64,
    slots: Vec<ArgumentOwnerSlot>,
}

thread_local! {
    static ARGUMENT_OWNER_FRAMES: RefCell<Vec<ArgumentOwnerFrame>> = const { RefCell::new(Vec::new()) };
}

static NEXT_FRAME_ID: AtomicU64 = AtomicU64::new(1);

/// RAII handle for a prepared caller frame. The callee consumes the matching
/// frame; this guard discards a still-pending frame on every other exit path.
#[must_use]
pub(crate) struct ArgumentOwnerFrameGuard {
    id: u64,
}

impl Drop for ArgumentOwnerFrameGuard {
    fn drop(&mut self) {
        ARGUMENT_OWNER_FRAMES.with(|frames| {
            let mut frames = frames.borrow_mut();
            if let Some(index) = frames.iter().rposition(|frame| frame.id == self.id) {
                frames.remove(index);
            }
        });
    }
}

/// Push one caller-owned frame. Frame slots borrow existing companions, so
/// this action never changes reference counts.
pub(crate) fn prepare_argument_owner_frame(
    slots: impl IntoIterator<Item = ArgumentOwnerSlot>,
) -> ArgumentOwnerFrameGuard {
    let id = NEXT_FRAME_ID.fetch_add(1, Ordering::Relaxed);
    ARGUMENT_OWNER_FRAMES.with(|frames| {
        frames.borrow_mut().push(ArgumentOwnerFrame {
            id,
            slots: slots.into_iter().collect(),
        });
    });
    ArgumentOwnerFrameGuard { id }
}

/// Consume exactly the top frame when all physical values match. A missing,
/// malformed, or mismatched frame is discarded and yields ownerless slots.
/// The caller/callee integration owns any retain needed to install a returned
/// owner into a callee companion.
pub(crate) fn consume_matching_argument_owners(values: &[MbValue]) -> Vec<MbValue> {
    ARGUMENT_OWNER_FRAMES.with(|frames| {
        let frame = frames.borrow_mut().pop();
        let Some(frame) = frame else {
            return vec![MbValue::none(); values.len()];
        };
        if frame.slots.len() != values.len()
            || frame
                .slots
                .iter()
                .zip(values)
                .any(|(slot, value)| slot.value_bits != value.to_bits())
        {
            return vec![MbValue::none(); values.len()];
        }
        frame.slots.into_iter().map(|slot| slot.owner).collect()
    })
}

/// Discard any abandoned frames during runtime teardown. Frames borrow caller
/// companions, so clearing this stack never releases a payload itself.
pub(crate) fn cleanup_argument_owner_frames() {
    ARGUMENT_OWNER_FRAMES.with(|frames| frames.borrow_mut().clear());
}

#[cfg(test)]
pub(crate) fn argument_owner_frame_depth() -> usize {
    ARGUMENT_OWNER_FRAMES.with(|frames| frames.borrow().len())
}

#[cfg(test)]
pub(crate) fn clear_argument_owner_frames_for_test() {
    cleanup_argument_owner_frames();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_frames_are_lifo_and_collision_safe() {
        clear_argument_owner_frames_for_test();
        let collision = MbValue::from_bits(0x0000_7fff_dead_beef);
        let bigint = crate::runtime::bigint_ops::bigint_from_i128(1i128 << 70);

        let outer = prepare_argument_owner_frame([ArgumentOwnerSlot::new(
            collision,
            MbValue::none(),
        )]);
        let inner = prepare_argument_owner_frame([ArgumentOwnerSlot::new(bigint, bigint)]);
        assert_eq!(argument_owner_frame_depth(), 2);

        assert_eq!(consume_matching_argument_owners(&[bigint]), vec![bigint]);
        assert_eq!(argument_owner_frame_depth(), 1);
        assert_eq!(
            consume_matching_argument_owners(&[collision]),
            vec![MbValue::none()]
        );
        assert_eq!(argument_owner_frame_depth(), 0);

        drop(inner);
        drop(outer);
        unsafe { crate::runtime::rc::release_if_ptr(bigint) };
    }
}

// marker: missing-generator:mamba-argument-owner-frame
// HANDWRITE-END
