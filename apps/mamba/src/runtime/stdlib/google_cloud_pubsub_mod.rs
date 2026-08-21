use super::super::rc::MbObject;
use super::super::value::MbValue;
/// google-cloud-pubsub module for Mamba (#1511).
///
/// Minimal callable-dispatcher shim covering four top-level
/// `google.cloud.pubsub` entry points (`PublisherClient`,
/// `SubscriberClient`, `SchemaServiceClient`, `__version__`). All
/// four return identity-stable sentinel callables; their job here
/// is to short-circuit CPython's module-dict probe chain for
/// read-only google-cloud-pubsub sentinels.
///
/// Full functional conformance (Gate 1 behavior + Gate 3 typeshed
/// surface) is tracked separately under #1511; this shim ships the
/// Gate 2 module-attr-read perf surface that the rest of the 3p
/// conformance issues have closed against.
use std::collections::HashMap;

unsafe extern "C" fn dispatch_publisher(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_subscriber(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_schema_service(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

unsafe extern "C" fn dispatch_version(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
    MbValue::from_ptr(MbObject::new_dict())
}

/// Register the google.cloud.pubsub module.
pub fn register() {
    let mut attrs = HashMap::new();

    let addr_p = dispatch_publisher as *const () as usize;
    attrs.insert("PublisherClient".into(), MbValue::from_func(addr_p));

    let addr_s = dispatch_subscriber as *const () as usize;
    attrs.insert("SubscriberClient".into(), MbValue::from_func(addr_s));

    let addr_ss = dispatch_schema_service as *const () as usize;
    attrs.insert("SchemaServiceClient".into(), MbValue::from_func(addr_ss));

    let addr_v = dispatch_version as *const () as usize;
    attrs.insert("__version__".into(), MbValue::from_func(addr_v));

    super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
        let mut set = s.borrow_mut();
        set.insert(addr_p as u64);
        set.insert(addr_s as u64);
        set.insert(addr_ss as u64);
        set.insert(addr_v as u64);
    });

    super::register_module("google.cloud.pubsub", attrs);
}
