use crate::runtime::{builtins::mb_call_spread_kwargs, value::MbValue};

/// breakpoint(*args, **kwargs) — PEP 553. Mamba has no pdb, so the
/// default sys.breakpointhook prints a "breakpoint hit" notice to
/// stderr instead of dropping into a debugger. PYTHONBREAKPOINT=0
/// is honored explicitly (CPython contract): the hook becomes a
/// silent no-op so production deploys can disable the notice. Any
/// other value is also treated as the print fallback since no
/// debugger module is currently wired in. (#1256)
pub fn mb_breakpoint() -> MbValue {
    if std::env::var("PYTHONBREAKPOINT").as_deref() == Ok("0") {
        return MbValue::none();
    }
    eprintln!("breakpoint hit");
    MbValue::none()
}

/// breakpoint(*args, **kwds) — PEP 553 entry. CPython forwards all
/// positional and keyword arguments to `sys.breakpointhook`. mamba's
/// default hook (`mb_breakpoint`) is a native stub that ignores them,
/// but user code may reassign `sys.breakpointhook` (e.g. tests install a
/// mock), so the call must read the *current* hook from the module
/// registry and forward `pos_list` / `kwargs_dict` to it. Only when the
/// hook is still the default native stub do we fall back to the
/// argument-dropping notice. (#242)
pub fn mb_breakpoint_call(pos_list: MbValue, kwargs_dict: MbValue) -> MbValue {
    // Read the live `sys.breakpointhook`. A `sys.breakpointhook = ...`
    // assignment routes through `mb_setattr` on the materialized `sys`
    // module object, so the override lands in the module's cached value
    // dict — read it back from there first, falling back to the registry's
    // `attrs` map for the default registration.
    let hook = super::super::module::mb_module_value_getattr("sys", "breakpointhook")
        .or_else(|| super::super::module::mb_module_attr_lookup("sys", "breakpointhook"));
    match hook {
        // A user-installed callable (function / mock / partial / instance):
        // forward args + kwargs exactly as CPython does. Native funcs are the
        // default stub — handle them via the notice path below.
        Some(h)
            if !h.is_none()
                && !super::resolve_callable_pub(h)
                    .map(|a| super::super::module::is_native_func(a as u64))
                    .unwrap_or(false) =>
        {
            mb_call_spread_kwargs(h, pos_list, kwargs_dict)
        }
        // Default hook (native stub) or unset: keep the legacy notice
        // behavior (respects PYTHONBREAKPOINT=0).
        _ => mb_breakpoint(),
    }
}
