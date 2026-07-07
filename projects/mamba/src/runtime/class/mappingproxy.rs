use super::*;

pub(super) fn class_namespace_mappingproxy(
    class_name: &str,
    type_fields: Option<&crate::runtime::rc::MbRwLock<crate::runtime::rc::InstanceFields>>,
) -> MbValue {
    if let Some(fields) = type_fields {
        if let Some(namespace) = fields
            .read()
            .ok()
            .and_then(|f| f.get("__mamba_type_namespace__").copied())
        {
            return crate::runtime::dict_ops::mappingproxy_from_mapping(namespace);
        }
    }
    let dict = crate::runtime::dict_ops::mb_dict_new();
    if let Some(fields) = type_fields {
        let entries: Vec<(String, MbValue)> = fields
            .read()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        for (k, v) in entries {
            let key = MbValue::from_ptr(MbObject::new_str(k));
            crate::runtime::dict_ops::mb_dict_setitem(dict, key, v);
        }
    }
    for (k, v, _from_method_table) in class_own_members(class_name) {
        let key = MbValue::from_ptr(MbObject::new_str(k));
        crate::runtime::dict_ops::mb_dict_setitem(dict, key, v);
    }
    crate::runtime::dict_ops::mappingproxy_from_mapping(dict)
}
