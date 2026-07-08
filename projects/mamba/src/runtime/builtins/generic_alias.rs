use super::super::rc::{InstanceFields, MbRwLock};
use super::{mb_repr, pep695_display_name, str_value, MbValue, ObjData};

pub(super) fn generic_alias_origin_args(
    class_name: &str,
    fields: &MbRwLock<InstanceFields>,
) -> Option<(MbValue, Vec<MbValue>)> {
    let (origin_key, args_key) = match class_name {
        "GenericAlias" => ("__origin__", "__args__"),
        "types.GenericAlias" => ("_origin", "_args"),
        _ => return None,
    };
    let (origin, args_val) = {
        let g = fields.read().ok()?;
        (*g.get(origin_key)?, *g.get(args_key)?)
    };
    let args: Vec<MbValue> = args_val
        .as_ptr()
        .map(|ptr| unsafe {
            match &(*ptr).data {
                ObjData::Tuple(items) => items.to_vec(),
                ObjData::List(lock) => lock.read().unwrap().iter().copied().collect(),
                _ => vec![args_val],
            }
        })
        .unwrap_or_else(|| vec![args_val]);
    Some((origin, args))
}

fn type_object_display_name(v: MbValue) -> Option<String> {
    let ptr = v.as_ptr()?;
    unsafe {
        match &(*ptr).data {
            ObjData::Instance { class_name, fields } if class_name == "type" => fields
                .read()
                .ok()
                .and_then(|f| f.get("__name__").copied())
                .and_then(str_value),
            _ => None,
        }
    }
}

fn generic_alias_repr_part(v: MbValue) -> String {
    if let Some(name) = pep695_display_name(v) {
        return name;
    }
    if let Some(name) = type_object_display_name(v) {
        return name;
    }
    str_value(mb_repr(v)).unwrap_or_default()
}

pub(super) fn generic_alias_repr_string(
    class_name: &str,
    fields: &MbRwLock<InstanceFields>,
) -> Option<String> {
    let (origin, args) = generic_alias_origin_args(class_name, fields)?;
    let name = generic_alias_repr_part(origin);
    let parts = args
        .into_iter()
        .map(generic_alias_repr_part)
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!("{name}[{parts}]"))
}
