use super::*;

fn invalid_generator_throw_arg_type(value: MbValue) -> &'static str {
    if value.is_none() {
        "NoneType"
    } else if value.as_int().is_some() {
        "int"
    } else if value.as_bool().is_some() {
        "bool"
    } else if value.as_float().is_some() {
        "float"
    } else if let Some(ptr) = value.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(_) => "str",
                ObjData::List(_) => "list",
                ObjData::Tuple(_) => "tuple",
                ObjData::Dict(_) => "dict",
                ObjData::Instance { class_name, .. } if class_name == "type" => "type",
                ObjData::Instance { .. } => "instance",
                _ => "object",
            }
        }
    } else {
        "object"
    }
}

fn raise_invalid_generator_throw_arg(value: MbValue) -> MbValue {
    crate::runtime::exception::mb_raise(
        MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
        MbValue::from_ptr(MbObject::new_str(format!(
            "exceptions must be classes or instances deriving from BaseException, not {}",
            invalid_generator_throw_arg_type(value),
        ))),
    );
    MbValue::none()
}

fn resolve_generator_throw_args(
    exc_type: MbValue,
    exc_msg: MbValue,
) -> Result<(String, String), MbValue> {
    if let Some(s) = extract_str(exc_type) {
        // Plain string type name is retained for mamba's legacy
        // `g.throw("TypeError", "msg")` lowering path.
        return Ok((s, extract_str(exc_msg).unwrap_or_default()));
    }

    if let Some(ptr) = exc_type.as_ptr() {
        unsafe {
            if let ObjData::Instance {
                ref class_name,
                ref fields,
            } = (*ptr).data
            {
                if class_name == "type" {
                    let fields_guard = fields.read().unwrap();
                    let Some(type_name) =
                        fields_guard.get("__name__").and_then(|v| extract_str(*v))
                    else {
                        return Err(raise_invalid_generator_throw_arg(exc_type));
                    };
                    if !crate::runtime::exception::is_subclass_of(&type_name, "BaseException") {
                        return Err(raise_invalid_generator_throw_arg(exc_type));
                    }
                    return Ok((type_name, extract_str(exc_msg).unwrap_or_default()));
                }

                if !crate::runtime::exception::is_subclass_of(class_name, "BaseException") {
                    return Err(raise_invalid_generator_throw_arg(exc_type));
                }

                let fields_guard = fields.read().unwrap();
                let msg = if !exc_msg.is_none() {
                    extract_str(exc_msg).unwrap_or_default()
                } else {
                    fields_guard
                        .get("message")
                        .and_then(|v| exception_message_str(*v))
                        .or_else(|| {
                            fields_guard
                                .get("args")
                                .and_then(|t| first_tuple_element(*t))
                                .and_then(exception_message_str)
                        })
                        .unwrap_or_default()
                };
                return Ok((class_name.clone(), msg));
            }
        }
    }

    Err(raise_invalid_generator_throw_arg(exc_type))
}

/// Dispatch method calls on generator handles (.send, .throw, .close).
pub(super) fn dispatch_generator_method(gen: MbValue, method: &str, args: MbValue) -> MbValue {
    let arg_list = extract_args_list(args);
    match method {
        "send" => {
            let value = arg_list.first().copied().unwrap_or(MbValue::none());
            crate::runtime::generator::mb_generator_send(gen, value)
        }
        "throw" => {
            // g.throw(ExcType, message) or g.throw(exc_instance)
            // CPython 3.12: throw(value) where value is an exception instance
            let exc_type = arg_list.first().copied().unwrap_or(MbValue::none());
            let exc_msg = arg_list.get(1).copied().unwrap_or(MbValue::none());
            let (type_str, msg_str) = match resolve_generator_throw_args(exc_type, exc_msg) {
                Ok(parts) => parts,
                Err(raised) => return raised,
            };
            let type_val = MbValue::from_ptr(MbObject::new_str(type_str));
            let msg_val = MbValue::from_ptr(MbObject::new_str(msg_str));
            crate::runtime::generator::mb_generator_throw(gen, type_val, msg_val)
        }
        "close" => {
            crate::runtime::generator::mb_generator_close(gen);
            MbValue::none()
        }
        "__iter__" => gen,
        "__await__" if crate::runtime::stdlib::types_mod::is_coroutine_generator(gen) => gen,
        "__next__" => crate::runtime::generator::mb_generator_next(gen),
        // Async-generator protocol: `async def f(): yield` is routed
        // through the sync generator path (see ast_to_hir.rs AsyncFnDef
        // arm), so the same handle must answer both sync and async
        // iteration methods. `await g.__anext__()` works because mb_await
        // on a non-coroutine value passes it through unchanged.
        "__aiter__" => gen,
        "__anext__" => {
            let val = crate::runtime::generator::mb_generator_next(gen);
            // Generator completion sets CURRENT_EXCEPTION to StopIteration;
            // CPython's async-iter protocol expects StopAsyncIteration
            // instead. Convert in-place so user code can
            // `except StopAsyncIteration:` cleanly.
            let pending = crate::runtime::exception::mb_get_exception();
            if !pending.is_none() {
                if let Some(ptr) = pending.as_ptr() {
                    let is_stop = unsafe {
                        matches!(
                            &(*ptr).data,
                            crate::runtime::rc::ObjData::Instance { class_name, .. }
                                if class_name == "StopIteration"
                        )
                    };
                    if is_stop {
                        crate::runtime::exception::mb_clear_exception();
                        crate::runtime::exception::mb_raise(
                            MbValue::from_ptr(MbObject::new_str("StopAsyncIteration".to_string())),
                            MbValue::from_ptr(MbObject::new_str(String::new())),
                        );
                        return MbValue::none();
                    }
                }
            }
            // Wrap the yielded value in a pre-completed coroutine so that
            // `await g.__anext__()` sees a coroutine-like awaitable rather than
            // a raw yielded value. Boxing the value into a coroutine marked
            // exhausted with `result = val` makes the await path unambiguous
            // and round-trips the value cleanly.
            let coro = crate::runtime::async_rt::mb_coroutine_new(
                MbValue::from_ptr(MbObject::new_str("__anext_value__".to_string())),
                MbValue::from_ptr(MbObject::new_list(Vec::new())),
            );
            crate::runtime::async_rt::mb_coroutine_complete(coro, val);
            coro
        }
        "aclose" => {
            crate::runtime::generator::mb_generator_close(gen);
            MbValue::none()
        }
        "asend" => {
            let value = arg_list.first().copied().unwrap_or(MbValue::none());
            crate::runtime::generator::mb_generator_send(gen, value)
        }
        "athrow" => {
            // Re-route to throw with the same arg shape — matches CPython
            // sync-vs-async generator equivalence in mamba's flattened model.
            let exc_type = arg_list.first().copied().unwrap_or(MbValue::none());
            let exc_msg = arg_list.get(1).copied().unwrap_or(MbValue::none());
            let (type_str, msg_str) = match resolve_generator_throw_args(exc_type, exc_msg) {
                Ok(parts) => parts,
                Err(raised) => return raised,
            };
            crate::runtime::generator::mb_generator_throw(
                gen,
                MbValue::from_ptr(MbObject::new_str(type_str)),
                MbValue::from_ptr(MbObject::new_str(msg_str)),
            )
        }
        _ => {
            crate::runtime::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "'generator' object has no attribute '{method}'"
                ))),
            );
            MbValue::none()
        }
    }
}

/// Dispatch method calls on coroutine handles (.send, .throw, .close).
pub(super) fn dispatch_coroutine_method(coro: MbValue, method: &str, args: MbValue) -> MbValue {
    let arg_list = extract_args_list(args);
    match method {
        "send" => {
            let value = arg_list.first().copied().unwrap_or(MbValue::none());
            crate::runtime::async_rt::mb_coroutine_send(coro, value)
        }
        "throw" => {
            let exc_type = arg_list.first().copied().unwrap_or(MbValue::none());
            let exc_msg = arg_list.get(1).copied().unwrap_or(MbValue::none());
            let (type_str, msg_str) = match resolve_generator_throw_args(exc_type, exc_msg) {
                Ok(parts) => parts,
                Err(raised) => return raised,
            };
            crate::runtime::async_rt::mb_coroutine_throw(
                coro,
                MbValue::from_ptr(MbObject::new_str(type_str)),
                MbValue::from_ptr(MbObject::new_str(msg_str)),
            )
        }
        "close" => crate::runtime::async_rt::mb_coroutine_close(coro),
        "__await__" => crate::runtime::async_rt::mb_coroutine_await_wrapper(coro),
        _ => {
            crate::runtime::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "'coroutine' object has no attribute '{method}'"
                ))),
            );
            MbValue::none()
        }
    }
}

pub(super) fn dispatch_coroutine_wrapper_method(
    wrapper: MbValue,
    method: &str,
    args: MbValue,
) -> MbValue {
    let Some(coro) = crate::runtime::async_rt::coroutine_wrapper_target(wrapper) else {
        return MbValue::none();
    };
    let arg_list = extract_args_list(args);
    match method {
        "__iter__" | "__await__" => {
            unsafe {
                crate::runtime::rc::retain_if_ptr(wrapper);
            }
            wrapper
        }
        "__next__" => crate::runtime::async_rt::mb_coroutine_send(coro, MbValue::none()),
        "send" => {
            let value = arg_list.first().copied().unwrap_or(MbValue::none());
            crate::runtime::async_rt::mb_coroutine_send(coro, value)
        }
        "throw" => {
            let exc_type = arg_list.first().copied().unwrap_or(MbValue::none());
            let exc_msg = arg_list.get(1).copied().unwrap_or(MbValue::none());
            let (type_str, msg_str) = match resolve_generator_throw_args(exc_type, exc_msg) {
                Ok(parts) => parts,
                Err(raised) => return raised,
            };
            crate::runtime::async_rt::mb_coroutine_throw(
                coro,
                MbValue::from_ptr(MbObject::new_str(type_str)),
                MbValue::from_ptr(MbObject::new_str(msg_str)),
            )
        }
        "close" => crate::runtime::async_rt::mb_coroutine_close(coro),
        "__repr__" | "__str__" => MbValue::from_ptr(MbObject::new_str(format!(
            "<coroutine_wrapper object at 0x{:x}>",
            wrapper.as_ptr().map(|p| p as usize).unwrap_or(0)
        ))),
        _ => {
            crate::runtime::exception::mb_raise(
                MbValue::from_ptr(MbObject::new_str("AttributeError".to_string())),
                MbValue::from_ptr(MbObject::new_str(format!(
                    "'coroutine_wrapper' object has no attribute '{method}'"
                ))),
            );
            MbValue::none()
        }
    }
}

/// Extract arguments from a list MbValue.
fn extract_args_list(args: MbValue) -> Vec<MbValue> {
    if let Some(ptr) = args.as_ptr() {
        unsafe {
            if let ObjData::List(ref lock) = (*ptr).data {
                return lock.read().unwrap().to_vec();
            }
        }
    }
    Vec::new()
}
