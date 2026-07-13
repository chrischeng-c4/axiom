// HANDWRITE-BEGIN gap="missing-generator:mamba-argument-owner-frame" tracker="#1451" reason="Thread-local frame identity, matching, and nested cleanup require a runtime transaction primitive."
use crate::runtime::value::MbValue;
use smallvec::SmallVec;
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU64, Ordering};

type OwnerSlots = SmallVec<[ArgumentOwnerSlot; 4]>;

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
    slots: OwnerSlots,
    expected_slots: usize,
}

thread_local! {
    static ARGUMENT_OWNER_FRAMES: RefCell<Vec<ArgumentOwnerFrame>> = const { RefCell::new(Vec::new()) };
    /// An ownerless dynamic call must not accidentally consume an outer
    /// generated frame whose raw data happens to match its argument.
    static OWNER_LOOKUP_SUPPRESSIONS: Cell<usize> = const { Cell::new(0) };
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

#[must_use]
pub(crate) struct OwnerLookupSuppressionGuard;

impl OwnerLookupSuppressionGuard {
    fn enter() -> Self {
        OWNER_LOOKUP_SUPPRESSIONS.with(|depth| depth.set(depth.get() + 1));
        Self
    }
}

/// Isolate an ownerless dynamic invocation from any generated caller frame.
/// The guard leaves that caller frame intact for its normal post-call cleanup.
pub(crate) fn suppress_argument_owner_lookup() -> OwnerLookupSuppressionGuard {
    OwnerLookupSuppressionGuard::enter()
}

impl Drop for OwnerLookupSuppressionGuard {
    fn drop(&mut self) {
        OWNER_LOOKUP_SUPPRESSIONS.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

#[must_use]
pub(crate) enum DynamicArgumentOwnerFrameGuard {
    Prepared(ArgumentOwnerFrameGuard),
    Ownerless(OwnerLookupSuppressionGuard),
}

/// Push one caller-owned frame. Frame slots borrow existing companions, so
/// this action never changes reference counts.
pub(crate) fn prepare_argument_owner_frame(
    slots: impl IntoIterator<Item = ArgumentOwnerSlot>,
) -> ArgumentOwnerFrameGuard {
    let slots: OwnerSlots = slots.into_iter().collect();
    let id = NEXT_FRAME_ID.fetch_add(1, Ordering::Relaxed);
    ARGUMENT_OWNER_FRAMES.with(|frames| {
        frames.borrow_mut().push(ArgumentOwnerFrame {
            id,
            expected_slots: slots.len(),
            slots,
        });
    });
    ArgumentOwnerFrameGuard { id }
}

/// Build a dynamic-call frame from boxed runtime values through the explicit
/// typed-Int owner sidecar. This is deliberately separate from static codegen:
/// dynamic dispatch already owns MbValues, while static calls read companion
/// slots directly.
pub(crate) fn prepare_dynamic_argument_owner_frame(
    values: &[MbValue],
) -> DynamicArgumentOwnerFrameGuard {
    // The explicit sidecar contract has exactly one owned dynamic input kind:
    // a heap BigInt. Small tagged ints and every other runtime value carry no
    // owner, so bypass the Vec/RefCell frame transaction for the common path.
    if values
        .iter()
        .all(|value| !crate::runtime::builtins::is_bigint_value(*value))
    {
        return DynamicArgumentOwnerFrameGuard::Ownerless(suppress_argument_owner_lookup());
    }
    DynamicArgumentOwnerFrameGuard::Prepared(prepare_argument_owner_frame(
        values.iter().copied().map(|value| {
            ArgumentOwnerSlot::new(
                value,
                crate::runtime::symbols::mb_typed_int_owner_or_none(value),
            )
        }),
    ))
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

/// Begin a streamed caller frame for generated call sites. Slots are appended
/// before the callee is entered, then discarded by the caller after return.
pub(crate) fn begin_argument_owner_frame(expected_slots: usize) {
    let id = NEXT_FRAME_ID.fetch_add(1, Ordering::Relaxed);
    ARGUMENT_OWNER_FRAMES.with(|frames| {
        frames.borrow_mut().push(ArgumentOwnerFrame {
            id,
            slots: OwnerSlots::with_capacity(expected_slots),
            expected_slots,
        });
    });
}

/// Add one explicitly sourced owner slot to the current generated call frame.
/// A malformed or overfull frame is discarded rather than being repurposed.
pub(crate) fn push_argument_owner_slot(value: MbValue, owner: MbValue) {
    ARGUMENT_OWNER_FRAMES.with(|frames| {
        let mut frames = frames.borrow_mut();
        let Some(frame) = frames.last_mut() else {
            return;
        };
        if frame.slots.len() >= frame.expected_slots {
            frames.pop();
            return;
        }
        frame.slots.push(ArgumentOwnerSlot::new(value, owner));
    });
}

/// Read one matching owner slot without consuming the surrounding caller
/// frame. The caller's post-call discard handles uninstrumented targets while
/// a nested invocation always works on its own top-of-stack frame.
pub(crate) fn matching_argument_owner_slot(index: usize, value: MbValue) -> MbValue {
    if OWNER_LOOKUP_SUPPRESSIONS.with(|depth| depth.get() != 0) {
        return MbValue::none();
    }
    ARGUMENT_OWNER_FRAMES.with(|frames| {
        let frames = frames.borrow();
        let Some(frame) = frames.last() else {
            return MbValue::none();
        };
        if frame.slots.len() != frame.expected_slots {
            return MbValue::none();
        }
        frame
            .slots
            .get(index)
            .filter(|slot| slot.value_bits == value.to_bits())
            .map(|slot| slot.owner)
            .unwrap_or_else(MbValue::none)
    })
}

/// Remove the current generated caller frame. No payload is released because
/// frame slots are borrowed from caller companions.
pub(crate) fn discard_argument_owner_frame() {
    ARGUMENT_OWNER_FRAMES.with(|frames| {
        frames.borrow_mut().pop();
    });
}

#[no_mangle]
pub extern "C" fn mb_argument_owner_frame_begin(expected_slots: i64) {
    begin_argument_owner_frame(usize::try_from(expected_slots).unwrap_or_default());
}

#[no_mangle]
pub extern "C" fn mb_argument_owner_frame_push(value_bits: i64, owner_bits: i64) {
    push_argument_owner_slot(
        MbValue::from_bits(value_bits as u64),
        MbValue::from_bits(owner_bits as u64),
    );
}

#[no_mangle]
pub extern "C" fn mb_argument_owner_frame_take(index: i64, value_bits: i64) -> i64 {
    usize::try_from(index)
        .ok()
        .map(|index| matching_argument_owner_slot(index, MbValue::from_bits(value_bits as u64)))
        .unwrap_or_else(MbValue::none)
        .to_bits() as i64
}

#[no_mangle]
pub extern "C" fn mb_argument_owner_frame_discard() {
    discard_argument_owner_frame();
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

    #[test]
    fn streamed_slots_require_exact_index_and_value_match() {
        clear_argument_owner_frames_for_test();
        let bigint = crate::runtime::bigint_ops::bigint_from_i128(1i128 << 70);
        let raw = MbValue::from_bits(0x0000_7fff_dead_beef);
        begin_argument_owner_frame(2);
        push_argument_owner_slot(bigint, bigint);
        push_argument_owner_slot(raw, MbValue::none());

        assert_eq!(matching_argument_owner_slot(0, bigint), bigint);
        assert!(matching_argument_owner_slot(0, raw).is_none());
        assert!(matching_argument_owner_slot(1, raw).is_none());
        discard_argument_owner_frame();
        assert_eq!(argument_owner_frame_depth(), 0);
        unsafe { crate::runtime::rc::release_if_ptr(bigint) };
    }

    #[test]
    fn ownerless_dynamic_frame_cannot_consume_an_outer_matching_slot() {
        clear_argument_owner_frames_for_test();
        let bigint = crate::runtime::bigint_ops::bigint_from_i128(1i128 << 70);
        let small = MbValue::from_int(7);
        let outer = prepare_argument_owner_frame([ArgumentOwnerSlot::new(small, bigint)]);

        let ownerless = prepare_dynamic_argument_owner_frame(&[small]);
        assert!(matching_argument_owner_slot(0, small).is_none());
        drop(ownerless);

        assert_eq!(matching_argument_owner_slot(0, small), bigint);
        drop(outer);
        unsafe { crate::runtime::rc::release_if_ptr(bigint) };
    }
}

// marker: missing-generator:mamba-argument-owner-frame
// HANDWRITE-END
