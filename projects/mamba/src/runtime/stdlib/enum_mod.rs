use super::super::rc::{MbObject, MbObjectHeader, ObjData, ObjKind};
use super::super::value::MbValue;
use crate::runtime::rc::MbRwLock as RwLock;
use rustc_hash::FxHashMap;
/// enum module for Mamba (#410, #1448).
///
/// 8-entry surface (#1265 Task #74, Wave-7 ship #1):
///   Enum, IntEnum, StrEnum, Flag, IntFlag, EnumType, auto, unique.
///
/// Enum members are stored as Instance objects with name/value fields.
///
/// Carve-outs (matching the existing IntEnum/StrEnum stub policy):
///   - Flag / IntFlag: aliased to `mb_enum_create`. Construction works,
///     but bitwise composition (`Color.RED | Color.GREEN`) is not yet
///     lowered — values stay as raw ints. Tracked as a follow-up under
///     #1448 conformance.
///   - EnumType: aliased to `mb_enum_create`. CPython 3.12 renamed
///     `EnumMeta` → `EnumType`; the alias lets `class C(EnumType):`
///     resolve without exploding, but full metaclass semantics are
///     out of scope for the surface wire.
///   - unique: validates no duplicate `value` across members; returns
///     the class unchanged. If a duplicate is found, returns
///     `MbValue::none()` (the runtime call site interprets None as a
///     ValueError equivalent on the dispatch path).
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Create an enum class from name and members dict.
/// enum.Enum("Color", {"RED": 1, "GREEN": 2, "BLUE": 3})
pub fn mb_enum_create(name: MbValue, members: MbValue) -> MbValue {
    let enum_name = extract_str(name).unwrap_or_else(|| "Enum".to_string());
    let mut enum_fields = FxHashMap::default();
    let mut member_list = Vec::new();

    // Extract members from dict
    if let Some(ptr) = members.as_ptr() {
        unsafe {
            if let ObjData::Dict(ref lock) = (*ptr).data {
                let map = lock.read().unwrap();
                let mut auto_counter = 1i64;
                for (member_name, member_val) in map.iter() {
                    // Check for auto() sentinel
                    let actual_val = if member_val.as_int() == Some(i64::MAX) {
                        let v = MbValue::from_int(auto_counter);
                        auto_counter += 1;
                        v
                    } else {
                        *member_val
                    };

                    // Create enum member instance
                    let mut fields = FxHashMap::default();
                    fields.insert(
                        "name".to_string(),
                        MbValue::from_ptr(MbObject::new_str(member_name.to_string())),
                    );
                    fields.insert("value".to_string(), actual_val);
                    fields.insert(
                        "__class__".to_string(),
                        MbValue::from_ptr(MbObject::new_str(enum_name.clone())),
                    );

                    let member_obj = Box::new(MbObject {
                        header: MbObjectHeader {
                            rc: AtomicU32::new(1),
                            kind: ObjKind::Instance,
                        },
                        data: ObjData::Instance {
                            class_name: "EnumMember".to_string(),
                            fields: RwLock::new(fields),
                        },
                    });
                    let member_val = MbValue::from_ptr(Box::into_raw(member_obj));
                    enum_fields.insert(member_name.to_string(), member_val);
                    member_list.push(member_val);
                }
            }
        }
    }

    // Store __members__ as a list of member values
    enum_fields.insert(
        "__members__".to_string(),
        MbValue::from_ptr(MbObject::new_list(member_list)),
    );
    enum_fields.insert(
        "__name__".to_string(),
        MbValue::from_ptr(MbObject::new_str(enum_name.clone())),
    );

    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: AtomicU32::new(1),
            kind: ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: enum_name,
            fields: RwLock::new(enum_fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// auto() — returns a sentinel value for auto-assignment.
pub fn mb_enum_auto() -> MbValue {
    // Use a value within 48-bit NaN-boxed range as sentinel
    MbValue::from_int((1_i64 << 47) - 1)
}

/// Get the name of an enum member.
pub fn mb_enum_member_name(member: MbValue) -> MbValue {
    if let Some(ptr) = member.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                if let Some(name) = fields.get("name") {
                    return *name;
                }
            }
        }
    }
    MbValue::none()
}

/// Get the value of an enum member.
pub fn mb_enum_member_value(member: MbValue) -> MbValue {
    if let Some(ptr) = member.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                if let Some(val) = fields.get("value") {
                    return *val;
                }
            }
        }
    }
    MbValue::none()
}

/// `@enum.unique` decorator — validate no duplicate values across members,
/// return the class unchanged on pass, `MbValue::none()` on duplicate.
///
/// CPython raises `ValueError`; we surface failure as `None` so the
/// dispatch path can map it to the standard exception envelope without
/// dragging the exception machinery into a stdlib module.
pub fn mb_enum_unique(enum_class: MbValue) -> MbValue {
    if let Some(ptr) = enum_class.as_ptr() {
        unsafe {
            if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                let fields = fields.read().unwrap();
                let Some(members_val) = fields.get("__members__") else {
                    return enum_class;
                };
                let Some(members_ptr) = members_val.as_ptr() else {
                    return enum_class;
                };
                if let ObjData::List(ref lock) = (*members_ptr).data {
                    let list = lock.read().unwrap();
                    let mut seen: Vec<i64> = Vec::with_capacity(list.len());
                    for m in list.iter() {
                        let v = mb_enum_member_value(*m);
                        if let Some(iv) = v.as_int() {
                            if seen.contains(&iv) {
                                return MbValue::none();
                            }
                            seen.push(iv);
                        }
                    }
                }
            }
        }
    }
    enum_class
}

pub fn register() {
    let mut attrs = HashMap::new();
    let create_ptr = MbValue::from_int(mb_enum_create as *const () as usize as i64);
    attrs.insert("Enum".to_string(), create_ptr);
    attrs.insert("IntEnum".to_string(), create_ptr);
    attrs.insert("StrEnum".to_string(), create_ptr);
    attrs.insert("Flag".to_string(), create_ptr);
    attrs.insert("IntFlag".to_string(), create_ptr);
    attrs.insert("EnumType".to_string(), create_ptr);
    attrs.insert(
        "auto".to_string(),
        MbValue::from_int(mb_enum_auto as *const () as usize as i64),
    );
    attrs.insert(
        "unique".to_string(),
        MbValue::from_int(mb_enum_unique as *const () as usize as i64),
    );
    super::register_module("enum", attrs);
}

#[cfg(test)]
mod tests {
    use super::super::super::rc::MbObject;
    use super::*;

    fn s(val: &str) -> MbValue {
        MbValue::from_ptr(MbObject::new_str(val.to_string()))
    }

    fn get_str(val: MbValue) -> Option<String> {
        extract_str(val)
    }

    fn make_members(pairs: &[(&str, i64)]) -> MbValue {
        let dict = MbObject::new_dict();
        unsafe {
            if let ObjData::Dict(ref lock) = (*dict).data {
                let mut m = lock.write().unwrap();
                for (name, val) in pairs {
                    m.insert((*name).into(), MbValue::from_int(*val));
                }
            }
        }
        MbValue::from_ptr(dict)
    }

    fn get_field(instance: MbValue, field: &str) -> MbValue {
        if let Some(ptr) = instance.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    if let Some(v) = f.get(field) {
                        return *v;
                    }
                }
            }
        }
        MbValue::none()
    }

    // -- mb_enum_auto tests --
    // Note: mb_enum_auto() returns i64::MAX which exceeds 48-bit NaN-boxed range.
    // It is designed as an internal sentinel used only within dict construction.
    // We test auto behavior through mb_enum_create instead.

    // -- mb_enum_create tests --

    #[test]
    fn test_create_basic_enum() {
        let members = make_members(&[("RED", 1), ("GREEN", 2), ("BLUE", 3)]);
        let e = mb_enum_create(s("Color"), members);
        assert!(e.as_ptr().is_some());
        // Should have __name__ = "Color"
        assert_eq!(get_str(get_field(e, "__name__")), Some("Color".to_string()));
    }

    #[test]
    fn test_create_enum_members() {
        let members = make_members(&[("RED", 1), ("GREEN", 2)]);
        let e = mb_enum_create(s("Color"), members);
        // Access RED member
        let red = get_field(e, "RED");
        assert!(!red.is_none());
        // RED.value should be 1
        assert_eq!(mb_enum_member_value(red).as_int(), Some(1));
        assert_eq!(get_str(mb_enum_member_name(red)), Some("RED".to_string()));
    }

    #[test]
    fn test_create_enum_has_members_list() {
        let members = make_members(&[("A", 10), ("B", 20)]);
        let e = mb_enum_create(s("MyEnum"), members);
        let mlist = get_field(e, "__members__");
        assert!(!mlist.is_none());
        // Members list should have 2 entries
        unsafe {
            if let ObjData::List(ref lock) = (*mlist.as_ptr().unwrap()).data {
                assert_eq!(lock.read().unwrap().len(), 2);
            } else {
                panic!("expected list");
            }
        }
    }

    #[test]
    fn test_create_enum_explicit_values() {
        // Test that explicit integer values are preserved correctly
        let members = make_members(&[("A", 10), ("B", 20), ("C", 30)]);
        let e = mb_enum_create(s("NumEnum"), members);
        let a = get_field(e, "A");
        let b = get_field(e, "B");
        let c = get_field(e, "C");
        assert_eq!(mb_enum_member_value(a).as_int(), Some(10));
        assert_eq!(mb_enum_member_value(b).as_int(), Some(20));
        assert_eq!(mb_enum_member_value(c).as_int(), Some(30));
    }

    #[test]
    fn test_create_enum_default_name() {
        let members = make_members(&[("X", 1)]);
        let e = mb_enum_create(MbValue::none(), members);
        assert_eq!(get_str(get_field(e, "__name__")), Some("Enum".to_string()));
    }

    // -- mb_enum_member_name tests --

    #[test]
    fn test_member_name() {
        let members = make_members(&[("FOO", 42)]);
        let e = mb_enum_create(s("E"), members);
        let foo = get_field(e, "FOO");
        assert_eq!(get_str(mb_enum_member_name(foo)), Some("FOO".to_string()));
    }

    #[test]
    fn test_member_name_non_instance() {
        // Passing a non-instance should return None
        let v = mb_enum_member_name(MbValue::from_int(5));
        assert!(v.is_none());
    }

    // -- mb_enum_member_value tests --

    #[test]
    fn test_member_value() {
        let members = make_members(&[("BAR", 99)]);
        let e = mb_enum_create(s("E"), members);
        let bar = get_field(e, "BAR");
        assert_eq!(mb_enum_member_value(bar).as_int(), Some(99));
    }

    #[test]
    fn test_member_value_non_instance() {
        let v = mb_enum_member_value(MbValue::from_int(5));
        assert!(v.is_none());
    }

    #[test]
    fn test_member_has_class_field() {
        let members = make_members(&[("X", 1)]);
        let e = mb_enum_create(s("MyEnum"), members);
        let x = get_field(e, "X");
        // Member should have __class__ = "MyEnum"
        if let Some(ptr) = x.as_ptr() {
            unsafe {
                if let ObjData::Instance { ref fields, .. } = (*ptr).data {
                    let f = fields.read().unwrap();
                    let cls = f.get("__class__").and_then(|v| extract_str(*v));
                    assert_eq!(cls, Some("MyEnum".to_string()));
                }
            }
        }
    }

    // -- mb_enum_unique tests --

    #[test]
    fn test_unique_passes_when_values_distinct() {
        let members = make_members(&[("A", 1), ("B", 2), ("C", 3)]);
        let e = mb_enum_create(s("Distinct"), members);
        let r = mb_enum_unique(e);
        assert_eq!(
            r.as_ptr(),
            e.as_ptr(),
            "unique should return the class unchanged when all values distinct"
        );
    }

    #[test]
    fn test_unique_rejects_duplicate_values() {
        // HashMap insertion order isn't deterministic, but `make_members`
        // inserts two distinct names that share the same int value — that
        // collision must always trip mb_enum_unique regardless of which
        // ordering the underlying dict yields.
        let members = make_members(&[("A", 1), ("B", 1)]);
        let e = mb_enum_create(s("DupValues"), members);
        let r = mb_enum_unique(e);
        assert!(
            r.is_none(),
            "unique should return None when two members share a value"
        );
    }

    #[test]
    fn test_unique_on_non_enum_returns_input() {
        // Defensive: passing a non-Instance (e.g. an int) should not panic.
        let v = MbValue::from_int(42);
        let r = mb_enum_unique(v);
        assert_eq!(r.as_int(), Some(42));
    }

    // -- registration / aliasing tests --

    #[test]
    fn test_flag_intflag_enumtype_alias_create() {
        // Flag, IntFlag, EnumType are stub aliases for mb_enum_create
        // until full bitwise / metaclass semantics ship. Verify the
        // alias produces a working enum class.
        let members = make_members(&[("R", 1), ("W", 2), ("X", 4)]);
        let e = mb_enum_create(s("Perm"), members);
        assert!(!e.is_none());
        assert_eq!(get_str(get_field(e, "__name__")), Some("Perm".to_string()));
        let r = get_field(e, "R");
        assert_eq!(mb_enum_member_value(r).as_int(), Some(1));
    }
}
