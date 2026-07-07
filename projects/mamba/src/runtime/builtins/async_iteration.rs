use crate::runtime::builtins::{raise_type_error, value_type_name};
use crate::runtime::rc::MbObject;
use crate::runtime::value::MbValue;

fn completed_coroutine(name: &str, value: MbValue) -> MbValue {
    let coro = crate::runtime::async_rt::mb_coroutine_new(
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    );
    crate::runtime::async_rt::mb_coroutine_complete(coro, value);
    coro
}

fn call_dunder_zero(obj: MbValue, name: &str) -> MbValue {
    crate::runtime::class::mb_call_method(
        obj,
        MbValue::from_ptr(MbObject::new_str(name.to_string())),
        MbValue::from_ptr(MbObject::new_list(Vec::new())),
    )
}

/// aiter(async_iterable) — drive the async-iteration protocol.
pub fn mb_aiter(iterable: MbValue) -> MbValue {
    if crate::runtime::class::mb_lookup_dunder(
        iterable,
        MbValue::from_ptr(MbObject::new_str("__aiter__".to_string())),
    )
    .is_none()
    {
        raise_type_error(format!(
            "aiter() requires an async iterable, got '{}'",
            value_type_name(iterable)
        ));
        return MbValue::none();
    }
    let async_iter = call_dunder_zero(iterable, "__aiter__");
    if crate::runtime::exception::current_exception_type().is_some() {
        return MbValue::none();
    }
    if crate::runtime::class::mb_lookup_dunder(
        async_iter,
        MbValue::from_ptr(MbObject::new_str("__anext__".to_string())),
    )
    .is_none()
    {
        raise_type_error(format!(
            "aiter() returned an object without __anext__ (got '{}')",
            value_type_name(async_iter)
        ));
        return MbValue::none();
    }
    async_iter
}

/// anext(async_iterator) — return the iterator's next awaitable.
pub fn mb_anext(async_iter: MbValue) -> MbValue {
    if crate::runtime::class::mb_lookup_dunder(
        async_iter,
        MbValue::from_ptr(MbObject::new_str("__anext__".to_string())),
    )
    .is_none()
    {
        raise_type_error(format!(
            "anext() requires an async iterator, got '{}'",
            value_type_name(async_iter)
        ));
        return MbValue::none();
    }
    call_dunder_zero(async_iter, "__anext__")
}

/// anext(async_iterator, default) — await one step now, map exhaustion to default,
/// then return a completed coroutine so `await anext(it, default)` still sees an
/// awaitable value.
pub fn mb_anext_default(async_iter: MbValue, default: MbValue) -> MbValue {
    let awaitable = mb_anext(async_iter);
    if crate::runtime::exception::current_exception_type().is_some() {
        return MbValue::none();
    }
    let value = crate::runtime::async_task::mb_await(awaitable);
    if crate::runtime::exception::current_exception_type().as_deref() == Some("StopAsyncIteration")
    {
        crate::runtime::exception::mb_clear_exception();
        return completed_coroutine("__anext_default__", default);
    }
    if crate::runtime::exception::current_exception_type().is_some() {
        return MbValue::none();
    }
    completed_coroutine("__anext_value__", value)
}
