use super::super::async_task::{
    mb_async_wait as rt_async_wait, mb_await as rt_await, mb_create_task as rt_create_task,
    mb_gather as rt_gather, mb_run_until_complete, mb_sleep as rt_sleep,
};
use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
/// asyncio module for Mamba — delegates to async_rt / async_task event loop.
///
/// Wires user-facing asyncio API (asyncio.run, asyncio.sleep, asyncio.gather,
/// asyncio.create_task, asyncio.wait, asyncio.shield) to the existing
/// coroutine runtime in `runtime::async_task`.
use std::collections::HashMap;

// ── Native dispatchers (C-ABI wrappers over typed Rust fns) ──

macro_rules! dispatch_unary {
    ($name:ident, $fn:expr) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

macro_rules! dispatch_variadic {
    ($name:ident, $fn:expr) => {
        unsafe extern "C" fn $name(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            let list = MbValue::from_ptr(MbObject::new_list(a.to_vec()));
            $fn(list)
        }
    };
}

dispatch_unary!(dispatch_run, mb_asyncio_run);
dispatch_unary!(dispatch_sleep, rt_sleep);
dispatch_unary!(dispatch_create_task, rt_create_task);
dispatch_unary!(dispatch_ensure_future, rt_create_task);
dispatch_unary!(dispatch_shield, mb_asyncio_shield);
dispatch_variadic!(dispatch_gather, rt_gather);

unsafe extern "C" fn dispatch_wait(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let tasks = a.get(0).copied().unwrap_or_else(MbValue::none);
    let timeout = a.get(1).copied().unwrap_or_else(MbValue::none);
    rt_async_wait(tasks, timeout)
}

unsafe extern "C" fn dispatch_wait_for(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let coro = a.get(0).copied().unwrap_or_else(MbValue::none);
    rt_await(coro)
}

pub fn register() {
    let mut attrs = HashMap::new();

    // Functions: real dispatchers backed by async runtime
    let dispatchers: Vec<(&str, usize)> = vec![
        ("run", dispatch_run as *const () as usize),
        ("sleep", dispatch_sleep as *const () as usize),
        ("create_task", dispatch_create_task as *const () as usize),
        (
            "ensure_future",
            dispatch_ensure_future as *const () as usize,
        ),
        ("gather", dispatch_gather as *const () as usize),
        ("wait", dispatch_wait as *const () as usize),
        ("wait_for", dispatch_wait_for as *const () as usize),
        ("shield", dispatch_shield as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Constants
    attrs.insert(
        "FIRST_COMPLETED".into(),
        MbValue::from_ptr(MbObject::new_str("FIRST_COMPLETED".into())),
    );
    attrs.insert(
        "FIRST_EXCEPTION".into(),
        MbValue::from_ptr(MbObject::new_str("FIRST_EXCEPTION".into())),
    );
    attrs.insert(
        "ALL_COMPLETED".into(),
        MbValue::from_ptr(MbObject::new_str("ALL_COMPLETED".into())),
    );

    super::register_module("asyncio", attrs);
}

/// asyncio.run(coro) — drive the event loop until coro completes.
pub fn mb_asyncio_run(coro: MbValue) -> MbValue {
    mb_run_until_complete(coro)
}

/// asyncio.shield(aw) — pass through (no cancellation semantics yet).
pub fn mb_asyncio_shield(aws: MbValue) -> MbValue {
    aws
}

// ── Legacy stubs kept for any existing in-tree callers ──
// TODO: remove once all call sites migrate to dispatch-based API.

pub fn mb_asyncio_Future() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            lock.write().unwrap().insert(
                "__type__".into(),
                MbValue::from_ptr(MbObject::new_str("Future".to_string())),
            );
        }
    }
    MbValue::from_ptr(dict)
}

pub fn mb_asyncio_Task() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        if let ObjData::Dict(ref lock) = (*dict).data {
            lock.write().unwrap().insert(
                "__type__".into(),
                MbValue::from_ptr(MbObject::new_str("Task".to_string())),
            );
        }
    }
    MbValue::from_ptr(dict)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shield_passthrough() {
        let input = MbValue::from_int(42);
        let v = mb_asyncio_shield(input);
        assert_eq!(v.as_int(), Some(42));
    }
}
