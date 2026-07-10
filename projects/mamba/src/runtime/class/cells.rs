use super::*;

const CLASSCELL_MARKER_PREFIX: &str = "__mamba_classcell__:";

pub fn mb_class_mark_classcell_required(class_name: MbValue) {
    let Some(name) = extract_str(class_name) else {
        return;
    };
    CLASSCELL_REQUIRED.with(|required| {
        required.borrow_mut().insert(name);
    });
}

pub fn mb_class_bind_classcell(class_name: MbValue, symbol_id: MbValue) {
    let Some(name) = extract_str(class_name) else {
        return;
    };
    CLASSCELL_REQUIRED.with(|required| {
        required.borrow_mut().insert(name.clone());
    });
    CLASSCELL_SYMBOL_IDS.with(|symbols| {
        symbols.borrow_mut().insert(name, symbol_id.to_bits() as i64);
    });
}

pub(super) fn classcell_required_for(class_name: &str) -> bool {
    CLASSCELL_REQUIRED.with(|required| required.borrow().contains(class_name))
}

pub(super) fn classcell_marker(class_name: &str) -> MbValue {
    MbValue::from_ptr(MbObject::new_str(format!(
        "{CLASSCELL_MARKER_PREFIX}{class_name}"
    )))
}

fn classcell_owner_from_value(value: MbValue) -> Option<String> {
    extract_str(value).and_then(|s| {
        s.strip_prefix(CLASSCELL_MARKER_PREFIX)
            .map(|owner| owner.to_string())
    })
}

pub(crate) fn record_classcell_value_for_type_new(marker: MbValue, class_value: MbValue) -> bool {
    let owner = classcell_owner_from_value(marker).filter(|owner| classcell_required_for(owner));
    let Some(owner) = owner else {
        if !matches!(
            crate::runtime::closure::mb_cell_compare_value(marker),
            crate::runtime::closure::CellCompareValue::NotACell
        ) {
            crate::runtime::closure::mb_cell_set(marker, class_value);
            return true;
        }
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "__classcell__ must be a nonlocal cell, not {}",
                crate::runtime::builtins::value_type_name(marker)
            ))),
        );
        return false;
    };
    unsafe {
        crate::runtime::rc::retain_if_ptr(class_value);
    }
    CLASSCELL_VALUES.with(|values| {
        if let Some(previous) = values.borrow_mut().insert(owner.clone(), class_value) {
            unsafe {
                crate::runtime::rc::release_if_ptr(previous);
            }
        }
    });
    if let Some(symbol_id) =
        CLASSCELL_SYMBOL_IDS.with(|symbols| symbols.borrow().get(&owner).copied())
    {
        crate::runtime::closure::mb_capture_cell_set_id(
            MbValue::from_bits(symbol_id as u64),
            class_value,
        );
    }
    true
}

pub(super) fn clear_classcell_state(class_name: &str) {
    CLASSCELL_REQUIRED.with(|required| {
        required.borrow_mut().remove(class_name);
    });
    CLASSCELL_SYMBOL_IDS.with(|symbols| {
        symbols.borrow_mut().remove(class_name);
    });
    CLASSCELL_VALUES.with(|values| {
        if let Some(value) = values.borrow_mut().remove(class_name) {
            unsafe {
                crate::runtime::rc::release_if_ptr(value);
            }
        }
    });
}

fn class_value_display(value: MbValue) -> Option<String> {
    resolve_class_name(value).map(|name| class_display_name(&name))
}

pub(super) fn validate_classcell_after_metaclass_new(
    class_name: &str,
    _namespace: MbValue,
    metaclass_result: MbValue,
) {
    if !classcell_required_for(class_name) {
        return;
    }
    let class_value = CLASSCELL_VALUES.with(|values| values.borrow().get(class_name).copied());
    if class_value.is_none() {
        let display_name = class_display_name(class_name);
        clear_classcell_state(class_name);
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("RuntimeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "__class__ not set defining '{display_name}'"
            ))),
        );
        return;
    }
    let class_value = class_value.unwrap();
    if class_value.to_bits() != metaclass_result.to_bits() {
        let actual = class_value_display(class_value)
            .unwrap_or_else(|| crate::runtime::builtins::value_type_name(class_value));
        let expected =
            class_value_display(metaclass_result).unwrap_or_else(|| class_display_name(class_name));
        clear_classcell_state(class_name);
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "__class__ set to {actual}, expected {expected}"
            ))),
        );
        return;
    }
    clear_classcell_state(class_name);
}
