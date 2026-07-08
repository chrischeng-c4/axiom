use super::super::rc::{MbObject, ObjData};
use super::super::value::MbValue;
use super::make_type_object;
use crate::runtime::class;
use crate::runtime::pep695;
use crate::runtime::stdlib::typing_mod;

pub(super) enum TypeUnionBuild {
    Value(MbValue),
    InvalidOperand,
    NotUnion,
}

/// PEP 604 `T1 | T2` — build a tuple representing the union if both
/// operands look like type values (built-in type-name strings, registered
/// user-class names, or an existing union tuple from a previous `|`).
/// Returns InvalidOperand when exactly one side is type-like, matching
/// CPython's TypeError for `int | 42`.
pub(super) fn mb_bitor_type_union(a: MbValue, b: MbValue) -> TypeUnionBuild {
    let mut left: Vec<MbValue> = Vec::new();
    let mut right: Vec<MbValue> = Vec::new();
    let left_ok = collect_union_operand(a, &mut left);
    let right_ok = collect_union_operand(b, &mut right);
    match (left_ok, right_ok) {
        (true, true) => {
            left.extend(right);
            let parts = dedupe_union_parts(left);
            if parts.len() == 1 {
                TypeUnionBuild::Value(parts[0])
            } else {
                TypeUnionBuild::Value(make_union_type_value(parts))
            }
        }
        (true, false) | (false, true) => TypeUnionBuild::InvalidOperand,
        (false, false) => TypeUnionBuild::NotUnion,
    }
}

fn collect_union_operand(v: MbValue, out: &mut Vec<MbValue>) -> bool {
    // PEP 604 shorthand: `T | None` means `T | type(None)`. The literal
    // None operand stands in for NoneType in the union.
    if v.is_none() {
        out.push(make_type_object("NoneType"));
        return true;
    }
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => {
                    if is_type_name(s) {
                        out.push(make_type_object(s));
                        return true;
                    }
                    false
                }
                ObjData::Tuple(items) => {
                    for &item in items.iter() {
                        if !collect_union_operand(item, out) {
                            return false;
                        }
                    }
                    true
                }
                ObjData::Instance { class_name, fields } if class_name == "UnionType" => {
                    let args = fields.read().ok().and_then(|f| f.get("__args__").copied());
                    if let Some(args) = args {
                        return collect_union_operand(args, out);
                    }
                    false
                }
                ObjData::Instance { class_name, fields } if class_name == "type" => {
                    let name = fields
                        .read()
                        .ok()
                        .and_then(|f| f.get("__name__").and_then(|v| type_name_from_value(*v)));
                    match name {
                        Some(name) if is_type_name(&name) => {
                            out.push(v);
                            true
                        }
                        _ => false,
                    }
                }
                // PEP 695: TypeVars and TypeAliasTypes are valid `|` operands
                // (`type R = R | None`, `T | int` bounds) — they join the
                // union as themselves.
                ObjData::Instance { class_name, .. } if pep695::is_pep695_class(class_name) => {
                    out.push(v);
                    true
                }
                // PEP 585/604: a generic alias (`list[int]`, `list[T]`) is a
                // valid union operand — `list[T] | int` joins it as a member.
                ObjData::Instance { class_name, .. } if class_name == "typing.Alias" => {
                    out.push(v);
                    true
                }
                _ => false,
            }
        }
    } else {
        false
    }
}

pub(crate) fn make_union_type_value(parts: Vec<MbValue>) -> MbValue {
    // __parameters__: the free TypeVars in the members (computed before `parts`
    // is moved into the __args__ tuple).
    let params = typing_mod::typevar_params_tuple(&parts);
    let args = MbValue::from_ptr(MbObject::new_tuple(parts));
    let inst = MbObject::new_instance("UnionType".to_string());
    unsafe {
        if let ObjData::Instance { fields, .. } = &(*inst).data {
            let mut f = fields.write().unwrap();
            f.insert("__args__".to_string(), args);
            f.insert("__parameters__".to_string(), params);
        }
    }
    MbValue::from_ptr(inst)
}

fn dedupe_union_parts(parts: Vec<MbValue>) -> Vec<MbValue> {
    let mut out = Vec::new();
    let mut seen = Vec::<String>::new();
    for part in parts {
        let key = type_name_from_value(part).unwrap_or_else(|| format!("{:016x}", part.to_bits()));
        if !seen.iter().any(|s| s == &key) {
            seen.push(key);
            out.push(part);
        }
    }
    out
}

fn type_name_from_value(v: MbValue) -> Option<String> {
    if let Some(ptr) = v.as_ptr() {
        unsafe {
            match &(*ptr).data {
                ObjData::Str(s) => Some(s.clone()),
                ObjData::Instance { class_name, fields } if class_name == "type" => {
                    fields.read().ok().and_then(|f| {
                        f.get("__name__")
                            .and_then(|name| type_name_from_value(*name))
                    })
                }
                _ => None,
            }
        }
    } else {
        None
    }
}

fn union_type_args(v: MbValue) -> Option<Vec<MbValue>> {
    v.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Instance { class_name, fields } = &(*ptr).data {
            if class_name == "UnionType" {
                return fields
                    .read()
                    .ok()
                    .and_then(|f| f.get("__args__").copied())
                    .and_then(|args| args.as_ptr())
                    .and_then(|args_ptr| match &(*args_ptr).data {
                        ObjData::Tuple(items) => Some(items.clone()),
                        _ => None,
                    });
            }
        }
        None
    })
}

pub(crate) fn union_type_repr(v: MbValue) -> String {
    let Some(args) = union_type_args(v) else {
        return "UnionType()".to_string();
    };
    args.iter()
        .map(|arg| match type_name_from_value(*arg).as_deref() {
            Some("NoneType") => "None".to_string(),
            Some(name) => name.to_string(),
            None => "...".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

pub(crate) fn is_type_name(s: &str) -> bool {
    matches!(
        s,
        "int"
            | "str"
            | "float"
            | "bool"
            | "list"
            | "dict"
            | "set"
            | "frozenset"
            | "tuple"
            | "bytes"
            | "bytearray"
            | "complex"
            | "type"
            | "object"
            | "NoneType"
            | "range"
            | "slice"
            | "super"
    ) || class::class_is_registered(s)
}
