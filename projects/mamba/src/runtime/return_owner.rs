// HANDWRITE-BEGIN gap="missing-generator:mamba-return-owner-frame" tracker="#1452" reason="Thread-local return-owner publication and matching require a stack-scoped runtime transaction."
use crate::runtime::value::MbValue;
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU64, Ordering};

/// One owned companion moved out of a JIT callee immediately before its
/// physical return. The data bits are retained only to reject stale or nested
/// return tokens; no owner classification is performed here.
#[derive(Debug)]
struct ReturnOwnerFrame {
    id: u64,
    value_bits: u64,
    owner: MbValue,
}

thread_local! {
    static RETURN_OWNER_FRAMES: RefCell<Vec<ReturnOwnerFrame>> = const { RefCell::new(Vec::new()) };
    static RETURN_OWNER_FRAME_COUNT: Cell<usize> = const { Cell::new(0) };
}

static NEXT_RETURN_OWNER_FRAME_ID: AtomicU64 = AtomicU64::new(1);

/// Publish the callee-owned companion immediately before a raw-or-boxed Int
/// return. The frame owns `owner` until a matching caller takes it or a
/// teardown/mismatch discards it.
pub(crate) fn publish_return_owner(value: MbValue, owner: MbValue) {
    // Inline values do not carry an ownership companion. Recording an empty
    // token would only force the caller through a RefCell transaction; actual
    // BigInt ownership remains explicitly stack-scoped below.
    if owner.is_none() {
        return;
    }
    let id = NEXT_RETURN_OWNER_FRAME_ID.fetch_add(1, Ordering::Relaxed);
    RETURN_OWNER_FRAMES.with(|frames| {
        frames.borrow_mut().push(ReturnOwnerFrame {
            id,
            value_bits: value.to_bits(),
            owner,
        });
    });
    RETURN_OWNER_FRAME_COUNT.with(|count| count.set(count.get() + 1));
}

/// Move the top owner into the immediate caller only when its returned data
/// bits match. A mismatch consumes and releases the top token so nested or
/// stale state can never be reused by a later dynamic call.
pub(crate) fn take_matching_return_owner(value: MbValue) -> MbValue {
    if RETURN_OWNER_FRAME_COUNT.with(|count| count.get() == 0) {
        return MbValue::none();
    }
    RETURN_OWNER_FRAMES.with(|frames| {
        let Some(frame) = frames.borrow_mut().pop() else {
            return MbValue::none();
        };
        RETURN_OWNER_FRAME_COUNT.with(|count| count.set(count.get().saturating_sub(1)));
        if frame.value_bits == value.to_bits() {
            frame.owner
        } else {
            unsafe { crate::runtime::rc::release_if_ptr(frame.owner) };
            MbValue::none()
        }
    })
}

/// Discard the current return token when a direct-call path aborts before it
/// can normalize the result. A published companion is owned by the frame and
/// must be released exactly once here.
pub(crate) fn discard_return_owner() {
    RETURN_OWNER_FRAMES.with(|frames| {
        if let Some(frame) = frames.borrow_mut().pop() {
            RETURN_OWNER_FRAME_COUNT.with(|count| count.set(count.get().saturating_sub(1)));
            unsafe { crate::runtime::rc::release_if_ptr(frame.owner) };
        }
    });
}

#[no_mangle]
pub extern "C" fn mb_return_owner_publish(value_bits: i64, owner_bits: i64) {
    publish_return_owner(
        MbValue::from_bits(value_bits as u64),
        MbValue::from_bits(owner_bits as u64),
    );
}

#[no_mangle]
pub extern "C" fn mb_return_owner_take(value_bits: i64) -> i64 {
    take_matching_return_owner(MbValue::from_bits(value_bits as u64)).to_bits() as i64
}

#[no_mangle]
pub extern "C" fn mb_return_owner_discard() {
    discard_return_owner();
}

/// Runtime teardown owns any abandoned published companions.
pub(crate) fn cleanup_return_owner_frames() {
    RETURN_OWNER_FRAMES.with(|frames| {
        for frame in frames.borrow_mut().drain(..) {
            unsafe { crate::runtime::rc::release_if_ptr(frame.owner) };
        }
    });
    RETURN_OWNER_FRAME_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn return_owner_frame_depth() -> usize {
    RETURN_OWNER_FRAMES.with(|frames| frames.borrow().len())
}

#[cfg(test)]
pub(crate) fn clear_return_owner_frames_for_test() {
    cleanup_return_owner_frames();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_returns_do_not_allocate_owner_tokens() {
        clear_return_owner_frames_for_test();
        let collision = MbValue::from_bits(0x0000_7fff_dead_beef);
        let bigint = crate::runtime::bigint_ops::bigint_from_i128(1i128 << 70);

        publish_return_owner(collision, MbValue::none());
        publish_return_owner(bigint, bigint);
        assert_eq!(return_owner_frame_depth(), 1);
        assert_eq!(take_matching_return_owner(bigint), bigint);
        assert_eq!(take_matching_return_owner(collision), MbValue::none());
        assert_eq!(return_owner_frame_depth(), 0);

        unsafe { crate::runtime::rc::release_if_ptr(bigint) };
    }

    #[test]
    fn mismatch_discards_the_owned_top_token() {
        clear_return_owner_frames_for_test();
        let bigint = crate::runtime::bigint_ops::bigint_from_i128(1i128 << 70);
        publish_return_owner(bigint, bigint);

        assert!(take_matching_return_owner(MbValue::from_int(7)).is_none());
        assert_eq!(return_owner_frame_depth(), 0);
    }
}

// marker: missing-generator:mamba-return-owner-frame
// HANDWRITE-END
