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

pub(crate) fn consume_classcell_marker_for_type_new(class_name: &str, value: MbValue) -> bool {
    let Some(owner) = classcell_owner_from_value(value) else {
        return true;
    };
    if owner != class_name {
        let owner_display = class_display_name(&owner);
        let class_display = class_display_name(class_name);
        crate::runtime::exception::mb_raise(
            MbValue::from_ptr(MbObject::new_str("TypeError".to_string())),
            MbValue::from_ptr(MbObject::new_str(format!(
                "__class__ set to {owner_display}, expected {class_display}"
            ))),
        );
        return false;
    }
    CLASSCELL_CONSUMED.with(|consumed| {
        consumed.borrow_mut().insert(owner);
    });
    true
}

fn classcell_owner_from_namespace(namespace: MbValue) -> Option<String> {
    namespace.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Dict(ref lock) = (*ptr).data {
            lock.read()
                .ok()
                .and_then(|map| {
                    map.get(&crate::runtime::dict_ops::DictKey::Str(
                        "__classcell__".to_string(),
                    ))
                    .copied()
                })
                .and_then(classcell_owner_from_value)
        } else {
            None
        }
    })
}

pub(super) fn clear_classcell_state(class_name: &str) {
    CLASSCELL_REQUIRED.with(|required| {
        required.borrow_mut().remove(class_name);
    });
    CLASSCELL_CONSUMED.with(|consumed| {
        consumed.borrow_mut().remove(class_name);
    });
}

pub(super) fn validate_classcell_after_metaclass_new(class_name: &str, namespace: MbValue) {
    if !classcell_required_for(class_name) {
        return;
    }
    let namespace_owner = classcell_owner_from_namespace(namespace);
    let consumed = CLASSCELL_CONSUMED.with(|consumed| consumed.borrow().contains(class_name));
    if namespace_owner.as_deref() != Some(class_name) && !consumed {
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
    clear_classcell_state(class_name);
}
