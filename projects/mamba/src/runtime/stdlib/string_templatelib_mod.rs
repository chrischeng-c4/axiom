use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use std::collections::HashMap;

const TEMPLATE_CLASS: &str = "TemplatelibTemplate";
const INTERPOLATION_CLASS: &str = "TemplatelibInterpolation";

fn new_str(s: impl Into<String>) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(s.into()))
}

fn new_list(items: Vec<MbValue>) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(items))
}

fn raise_type_error(msg: impl Into<String>) -> MbValue {
    super::super::exception::mb_raise(new_str("TypeError"), new_str(msg.into()));
    MbValue::none()
}

fn make_instance(class_name: &str, fields: Vec<(&str, MbValue)>) -> MbValue {
    let inst = MbObject::new_instance(class_name.to_string());
    unsafe {
        if let ObjData::Instance {
            fields: ref iflds, ..
        } = (*inst).data
        {
            let mut guard = iflds.write().unwrap();
            for (key, value) in fields {
                super::super::rc::retain_if_ptr(value);
                guard.insert(key.to_string(), value);
            }
        }
    }
    MbValue::from_ptr(inst)
}

fn instance_class_name(value: MbValue) -> Option<String> {
    value.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { ref class_name, .. } = (*ptr).data {
            Some(class_name.clone())
        } else {
            None
        }
    })
}

fn is_template(value: MbValue) -> bool {
    instance_class_name(value).is_some_and(|name| {
        name == TEMPLATE_CLASS
            || super::super::class::class_mro_any(&name, |base| base == TEMPLATE_CLASS)
    })
}

unsafe extern "C" fn dispatch_template(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    crate::icf_guard!();
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    make_instance(TEMPLATE_CLASS, vec![("items", new_list(args.to_vec()))])
}

unsafe extern "C" fn dispatch_interpolation(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    crate::icf_guard!();
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let value = args.first().copied().unwrap_or_else(MbValue::none);
    let expression = args.get(1).copied().unwrap_or_else(|| new_str(""));
    let conversion = args.get(2).copied().unwrap_or_else(MbValue::none);
    let format_spec = args.get(3).copied().unwrap_or_else(|| new_str(""));
    make_instance(
        INTERPOLATION_CLASS,
        vec![
            ("value", value),
            ("expression", expression),
            ("conversion", conversion),
            ("format_spec", format_spec),
        ],
    )
}

unsafe extern "C" fn dispatch_convert(args_ptr: *const MbValue, nargs: usize) -> MbValue {
    let args = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
    let obj = args.first().copied().unwrap_or_else(MbValue::none);
    let conversion = args.get(1).copied().unwrap_or_else(MbValue::none);
    if conversion.is_none() {
        return obj;
    }
    if let Some(text) = conversion.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref value) = (*ptr).data {
            Some(value.clone())
        } else {
            None
        }
    }) {
        return match text.as_str() {
            "a" => super::super::builtins::mb_ascii(obj),
            "r" => super::super::builtins::mb_repr(obj),
            "s" => super::super::builtins::mb_str(obj),
            _ => raise_type_error("convert() conversion must be 'a', 'r', 's', or None"),
        };
    }
    raise_type_error(format!(
        "convert() conversion must be str or None, not {}",
        super::super::builtins::value_type_name(conversion)
    ))
}

extern "C" fn m_template_add(this: MbValue, other: MbValue) -> MbValue {
    if !is_template(other) {
        return raise_type_error(format!(
            "can only concatenate Template (not \"{}\") to Template",
            super::super::builtins::value_type_name(other)
        ));
    }
    make_instance(TEMPLATE_CLASS, vec![("left", this), ("right", other)])
}

extern "C" fn m_template_iter(_this: MbValue) -> MbValue {
    super::super::iter::mb_iter(new_list(vec![]))
}

extern "C" fn m_template_values(_this: MbValue) -> MbValue {
    new_list(vec![])
}

extern "C" fn m_interpolation_new(_this: MbValue, args_list: MbValue) -> MbValue {
    let args = super::super::builtins::extract_items(args_list);
    if args.len() == 1 {
        return raise_type_error(
            "Interpolation.__new__() requires expression, conversion, and format_spec",
        );
    }
    let value = args.first().copied().unwrap_or_else(MbValue::none);
    let expression = args.get(1).copied().unwrap_or_else(|| new_str(""));
    let conversion = args.get(2).copied().unwrap_or_else(MbValue::none);
    let format_spec = args.get(3).copied().unwrap_or_else(|| new_str(""));
    if !expression
        .as_ptr()
        .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Str(_)) })
    {
        return raise_type_error("Interpolation.__new__() expression must be str");
    }
    if !format_spec
        .as_ptr()
        .is_some_and(|ptr| unsafe { matches!((*ptr).data, ObjData::Str(_)) })
    {
        return raise_type_error("Interpolation.__new__() format_spec must be str");
    }
    make_instance(
        INTERPOLATION_CLASS,
        vec![
            ("value", value),
            ("expression", expression),
            ("conversion", conversion),
            ("format_spec", format_spec),
        ],
    )
}

fn register_classes() {
    let mut template_methods = HashMap::new();
    template_methods.insert(
        "__add__".to_string(),
        MbValue::from_func(m_template_add as *const () as usize),
    );
    template_methods.insert(
        "__iter__".to_string(),
        MbValue::from_func(m_template_iter as *const () as usize),
    );
    template_methods.insert(
        "values".to_string(),
        MbValue::from_func(m_template_values as *const () as usize),
    );
    super::super::class::mb_class_register(
        TEMPLATE_CLASS,
        vec!["object".to_string()],
        template_methods,
    );

    let mut interpolation_methods = HashMap::new();
    interpolation_methods.insert(
        "__new__".to_string(),
        MbValue::from_func(m_interpolation_new as *const () as usize),
    );
    super::super::module::register_variadic_func(m_interpolation_new as *const () as u64);
    super::super::class::mb_class_register(
        INTERPOLATION_CLASS,
        vec!["object".to_string()],
        interpolation_methods,
    );
}

fn register_ctor(attrs: &mut HashMap<String, MbValue>, name: &str, addr: usize, class_name: &str) {
    attrs.insert(name.to_string(), MbValue::from_func(addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|funcs| {
        funcs.borrow_mut().insert(addr as u64);
    });
    super::super::module::register_native_type_name(addr as u64, class_name.to_string());
}

fn attach_to_string_parent() {
    let child = super::super::module::MODULES.with(|mods| {
        mods.borrow_mut()
            .get_mut("string.templatelib")
            .map(super::super::module::module_to_value_and_cache)
    });
    if let Some(child) = child {
        super::super::module::mb_module_setattr(new_str("string"), new_str("templatelib"), child);
    }
}

pub fn register() {
    register_classes();

    let mut attrs = HashMap::new();
    register_ctor(
        &mut attrs,
        "Template",
        dispatch_template as *const () as usize,
        TEMPLATE_CLASS,
    );
    register_ctor(
        &mut attrs,
        "Interpolation",
        dispatch_interpolation as *const () as usize,
        INTERPOLATION_CLASS,
    );
    let convert_addr = dispatch_convert as *const () as usize;
    attrs.insert("convert".to_string(), MbValue::from_func(convert_addr));
    super::super::module::NATIVE_FUNC_ADDRS.with(|funcs| {
        funcs.borrow_mut().insert(convert_addr as u64);
    });

    super::register_module("string.templatelib", attrs);
    attach_to_string_parent();
}
