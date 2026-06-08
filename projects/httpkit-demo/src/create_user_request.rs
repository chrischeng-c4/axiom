// SPEC-MANAGED: .aw/tech-design/projects/httpkit-demo/create-user-request.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Request body for `POST /users`. Validated pydantic-like.
/// @spec .aw/tech-design/projects/httpkit-demo/create-user-request.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateUserRequest {
    /// Display name. 1–64 chars.
    pub name: String,
    /// User email. Must contain '@'. No further RFC 5322 check in this demo.
    pub email: String,
    /// Age in years. Sanity-bounded to avoid obvious bogus inputs.
    pub age: i64,
}

/// @spec .aw/tech-design/projects/httpkit-demo/create-user-request.md#x-constructor
impl CreateUserRequest {
    /// Constructor generated from `x-constructor`. Returns `Err(String)` when
    /// a validation rule fails.
    pub fn new(name: String, email: String, age: i64) -> Result<Self, String> {
        if !(name.len() >= 1) {
            return Err(format!("name must be non-empty"));
        }
        if !(name.len() <= 64) {
            return Err(format!("name too long (max 64 chars)"));
        }
        if !(email.contains('@')) {
            return Err(format!("email must contain '@'"));
        }
        if !((0..=150).contains(&(age as i64))) {
            return Err(format!("age out of range [0, 150]"));
        }
        Ok(Self { name, email, age })
    }
}

/// Mamba FFI entry point. Generated from `x-mamba-binding.extern_fn`.
/// @spec .aw/tech-design/projects/httpkit-demo/create-user-request.md#x-mamba-binding
#[no_mangle]
pub unsafe extern "C" fn create_user_request_new(
    args: *const cclab_mamba_registry::MbValue,
    nargs: usize,
) -> cclab_mamba_registry::MbValue {
    use cclab_mamba_registry::{convert::mb_wrap_native, FromMbValue, MbValue};

    // Positional arg reader — returns `MbValue::none()` when index is out of range.
    let read = |i: usize| -> MbValue {
        if i < nargs {
            unsafe { *args.add(i) }
        } else {
            MbValue::none()
        }
    };
    let name: String = String::from_mb_value(read(0))
        .ok()
        .unwrap_or_else(|| Default::default());
    let email: String = String::from_mb_value(read(1))
        .ok()
        .unwrap_or_else(|| Default::default());
    let age: i64 = i64::from_mb_value(read(2))
        .ok()
        .map(|v| v)
        .unwrap_or_else(|| 0);

    match CreateUserRequest::new(name, email, age) {
        Ok(value) => mb_wrap_native(value),
        Err(msg) => {
            // Raise ValueError via the mamba ops table when installed. The
            // `OBJECT_OPS.get()` path is a graceful no-op in unit tests where
            // no mamba runtime has called `set_object_ops` — callers of the
            // FFI shim under test just observe `MbValue::none()`.
            if let Some(o) = cclab_mamba_registry::ops::OBJECT_OPS.get() {
                (o.raise)("ValueError", &msg);
            }
            MbValue::none()
        }
    }
}

/// Attribute getter for `CreateUserRequest.name`. Auto-generated from
/// `x-mamba-attributes`.
/// @spec .aw/tech-design/projects/httpkit-demo/create-user-request.md#x-mamba-attributes.name
#[no_mangle]
pub unsafe extern "C" fn create_user_request_get_name(
    args: *const cclab_mamba_registry::MbValue,
    _nargs: usize,
) -> cclab_mamba_registry::MbValue {
    use cclab_mamba_registry::{convert::mb_unwrap_native_ref, IntoMbValue, MbValue};
    let self_: &CreateUserRequest = match unsafe { mb_unwrap_native_ref(*args) } {
        Some(r) => r,
        None => return MbValue::none(),
    };
    let value = { self_.name.clone() };
    value.into_mb_value()
}

/// Attribute getter for `CreateUserRequest.email`. Auto-generated from
/// `x-mamba-attributes`.
/// @spec .aw/tech-design/projects/httpkit-demo/create-user-request.md#x-mamba-attributes.email
#[no_mangle]
pub unsafe extern "C" fn create_user_request_get_email(
    args: *const cclab_mamba_registry::MbValue,
    _nargs: usize,
) -> cclab_mamba_registry::MbValue {
    use cclab_mamba_registry::{convert::mb_unwrap_native_ref, IntoMbValue, MbValue};
    let self_: &CreateUserRequest = match unsafe { mb_unwrap_native_ref(*args) } {
        Some(r) => r,
        None => return MbValue::none(),
    };
    let value = { self_.email.clone() };
    value.into_mb_value()
}

/// Attribute getter for `CreateUserRequest.age`. Auto-generated from
/// `x-mamba-attributes`.
/// @spec .aw/tech-design/projects/httpkit-demo/create-user-request.md#x-mamba-attributes.age
#[no_mangle]
pub unsafe extern "C" fn create_user_request_get_age(
    args: *const cclab_mamba_registry::MbValue,
    _nargs: usize,
) -> cclab_mamba_registry::MbValue {
    use cclab_mamba_registry::{convert::mb_unwrap_native_ref, IntoMbValue, MbValue};
    let self_: &CreateUserRequest = match unsafe { mb_unwrap_native_ref(*args) } {
        Some(r) => r,
        None => return MbValue::none(),
    };
    let value = { self_.age };
    value.into_mb_value()
}

/// Register the `CreateUserRequest` symbol into a [`cclab_mamba_registry::ModuleRegistrar`].
///
/// Named `register_create_user_request` so multiple types emitted from a single
/// spec (via JSON Schema `definitions`) do not collide. An aggregate
/// `pub fn register(r)` — emitted at the end of the file — calls each of
/// these per-type registrars, and is what the `auto_wire_mamba_lib`
/// post-pass looks for when wiring `lib.rs`.
/// @spec .aw/tech-design/projects/httpkit-demo/create-user-request.md#x-mamba-binding.register
pub fn register_create_user_request(r: &mut cclab_mamba_registry::ModuleRegistrar) {
    use cclab_mamba_registry::rt_sym;
    r.add_symbol(rt_sym!(
        "CreateUserRequest",
        create_user_request_new,
        "CreateUserRequest(name: str, email: str, age: int) -> CreateUserRequest"
    ));

    // Per-attribute getters — stored in the ops table today, dispatched by
    // `getattr` after mamba PR-5 lands. Registering here is harmless until
    // then (the callbacks are just held in `ATTRIBUTE_GETTERS`).
    if let Some(o) = cclab_mamba_registry::ops::OBJECT_OPS.get() {
        (o.register_getter)("CreateUserRequest", "name", create_user_request_get_name);
        (o.register_getter)("CreateUserRequest", "email", create_user_request_get_email);
        (o.register_getter)("CreateUserRequest", "age", create_user_request_get_age);
    }
}

/// Aggregate registrar for every type this spec declares. Consumed by
/// `apply.rs::auto_wire_mamba_lib` to wire a single `register(r)` call
/// into the owning crate's `MambaModule::register` body.
/// @spec .aw/tech-design/projects/httpkit-demo/create-user-request.md#x-mamba-binding.register-aggregate
pub fn register(r: &mut cclab_mamba_registry::ModuleRegistrar) {
    register_create_user_request(r);
}

// CODEGEN-END
