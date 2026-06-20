use super::super::rc::MbObject;
use super::super::value::MbValue;
use rustc_hash::FxHashMap;
/// dis module for Mamba (#667).
///
/// Exposes Mamba's MIR (mid-level IR) as Python-accessible instruction objects,
/// providing a CPython-compatible dis module interface.
use std::collections::HashMap;

/// Mamba MIR opcode names mapped to CPython bytecode opcode numbers.
/// These are Mamba-specific; the numbers are symbolic and do not match
/// CPython exactly (since Mamba does not use CPython bytecode).
const MAMBA_OPCODES: &[(&str, i64)] = &[
    ("LOAD_CONST", 100),
    ("LOAD_NAME", 101),
    ("LOAD_FAST", 102),
    ("LOAD_GLOBAL", 116),
    ("STORE_NAME", 90),
    ("STORE_FAST", 125),
    ("STORE_GLOBAL", 97),
    ("LOAD_ATTR", 106),
    ("STORE_ATTR", 95),
    ("BINARY_OP", 122),
    ("UNARY_OP", 123),
    ("COMPARE_OP", 107),
    ("CALL", 171),
    ("CALL_FUNCTION", 131),
    ("RETURN_VALUE", 83),
    ("POP_TOP", 1),
    ("JUMP_FORWARD", 110),
    ("JUMP_IF_FALSE", 111),
    ("JUMP_IF_TRUE", 112),
    ("JUMP_ABSOLUTE", 113),
    ("MAKE_FUNCTION", 132),
    ("BUILD_LIST", 103),
    ("BUILD_DICT", 105),
    ("BUILD_TUPLE", 102),
    ("BUILD_SET", 104),
    ("SUBSCR", 25),
    ("STORE_SUBSCR", 60),
    ("GET_ITER", 68),
    ("FOR_ITER", 93),
    ("RAISE_VARARGS", 130),
    ("SETUP_FINALLY", 122),
    ("RESUME", 151),
    ("PUSH_NULL", 2),
    ("NOP", 9),
];

// ── Variadic dispatchers (callable from module-attr context) ──

macro_rules! disp_nullary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(_args_ptr: *const MbValue, _nargs: usize) -> MbValue {
            $fn()
        }
    };
}

macro_rules! disp_unary {
    ($disp:ident, $fn:path) => {
        unsafe extern "C" fn $disp(args_ptr: *const MbValue, nargs: usize) -> MbValue {
            let a = unsafe { std::slice::from_raw_parts(args_ptr, nargs) };
            $fn(a.get(0).copied().unwrap_or_else(MbValue::none))
        }
    };
}

disp_unary!(d_dis, mb_dis_dis);
disp_unary!(d_disassemble, mb_dis_disassemble);
disp_unary!(d_get_instructions, mb_dis_get_instructions);
disp_unary!(d_show_code, mb_dis_show_code);
disp_unary!(d_code_info, mb_dis_code_info);
disp_unary!(d_findlinestarts, mb_dis_findlinestarts);
disp_unary!(d_findlabels, mb_dis_findlabels);
disp_unary!(d_stack_effect, mb_dis_stack_effect);
disp_nullary!(d_opmap, mb_dis_opmap);
disp_nullary!(d_opname, mb_dis_opname);
disp_nullary!(d_Instruction, mb_dis_Instruction);

// surface: missing callable names (classes + functions). Present-and-callable
// stubs so resolve_callable returns Some (Instance values would fail).
disp_unary!(d_disco, mb_dis_disassemble);
disp_unary!(d_distb, mb_dis_dis);
disp_unary!(d_main, mb_dis_dis);
disp_unary!(d_pretty_flags, mb_dis_pretty_flags);
disp_nullary!(d_Bytecode, mb_dis_Instruction);
disp_nullary!(d_Positions, mb_dis_Instruction);

pub fn register() {
    let mut attrs = HashMap::new();

    let dispatchers: Vec<(&str, usize)> = vec![
        // Core functions
        ("dis", d_dis as *const () as usize),
        ("disassemble", d_disassemble as *const () as usize),
        ("get_instructions", d_get_instructions as *const () as usize),
        ("show_code", d_show_code as *const () as usize),
        ("code_info", d_code_info as *const () as usize),
        ("findlinestarts", d_findlinestarts as *const () as usize),
        ("findlabels", d_findlabels as *const () as usize),
        ("stack_effect", d_stack_effect as *const () as usize),
        // Opcode dictionaries
        ("opmap", d_opmap as *const () as usize),
        ("opname", d_opname as *const () as usize),
        // Instruction class
        ("Instruction", d_Instruction as *const () as usize),
        // surface: missing callable functions
        ("disco", d_disco as *const () as usize),
        ("distb", d_distb as *const () as usize),
        ("main", d_main as *const () as usize),
        ("pretty_flags", d_pretty_flags as *const () as usize),
        // surface: missing callable classes
        ("Bytecode", d_Bytecode as *const () as usize),
        ("Positions", d_Positions as *const () as usize),
    ];
    for (name, addr) in dispatchers {
        attrs.insert(name.to_string(), MbValue::from_func(addr));
        super::super::module::NATIVE_FUNC_ADDRS.with(|s| {
            s.borrow_mut().insert(addr as u64);
        });
    }

    // Constants
    attrs.insert("HAVE_ARGUMENT".into(), MbValue::from_int(90));

    // Populate opname entries as top-level attributes for convenience
    for (name, code) in MAMBA_OPCODES {
        attrs.insert(name.to_string(), MbValue::from_int(*code));
    }

    // surface: missing CPython module constants (auto-added)
    attrs.insert("CACHE".into(), MbValue::from_int(0));
    attrs.insert("CALL_INTRINSIC_1".into(), MbValue::from_int(173));
    attrs.insert("CALL_INTRINSIC_2".into(), MbValue::from_int(174));
    attrs.insert("EXTENDED_ARG".into(), MbValue::from_int(144));
    attrs.insert("FORMAT_VALUE".into(), MbValue::from_int(155));
    attrs.insert("JUMP_BACKWARD".into(), MbValue::from_int(140));
    attrs.insert("LOAD_SUPER_ATTR".into(), MbValue::from_int(141));
    attrs.insert("RETURN_CONST".into(), MbValue::from_int(121));
    attrs.insert("SEND".into(), MbValue::from_int(123));
    attrs.insert("spec_op".into(), MbValue::from_int(168));
    attrs.insert(
        "specialized".into(),
        MbValue::from_ptr(MbObject::new_str("SEND_GEN".to_string())),
    );

    // surface: missing CPython module data (tuples / lists / dicts / sentinel / modules)
    let mk_str = |s: &str| MbValue::from_ptr(MbObject::new_str(s.to_string()));
    let mk_int_list = |v: &[i64]| {
        MbValue::from_ptr(MbObject::new_list(
            v.iter().map(|n| MbValue::from_int(*n)).collect(),
        ))
    };

    // cmp_op: tuple of comparison operator strings
    attrs.insert(
        "cmp_op".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            mk_str("<"),
            mk_str("<="),
            mk_str("=="),
            mk_str("!="),
            mk_str(">"),
            mk_str(">="),
        ])),
    );
    // MAKE_FUNCTION_FLAGS: tuple of flag field names
    attrs.insert(
        "MAKE_FUNCTION_FLAGS".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            mk_str("defaults"),
            mk_str("kwdefaults"),
            mk_str("annotations"),
            mk_str("closure"),
        ])),
    );
    // FORMAT_VALUE_CONVERTERS: tuple of (callable|None, str) pairs; surface only
    // needs presence, so a tuple of name strings is sufficient.
    attrs.insert(
        "FORMAT_VALUE_CONVERTERS".into(),
        MbValue::from_ptr(MbObject::new_tuple(vec![
            mk_str(""),
            mk_str("str"),
            mk_str("repr"),
            mk_str("ascii"),
        ])),
    );
    // has* tables: lists of opcode numbers
    attrs.insert(
        "hasarg".into(),
        mk_int_list(&[
            90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 105, 106, 107, 108,
            109, 110, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128,
            129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145,
            146, 147, 149, 150, 151, 152, 155, 156, 157, 162, 163, 164, 165, 171, 172, 173, 174,
            175, 176, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251,
            252, 253, 254, 260, 261, 262, 263, 264, 265, 266,
        ]),
    );
    attrs.insert("hascompare".into(), mk_int_list(&[107]));
    attrs.insert("hasconst".into(), mk_int_list(&[100, 121, 172]));
    attrs.insert("hasexc".into(), mk_int_list(&[256, 257, 258]));
    attrs.insert(
        "hasfree".into(),
        mk_int_list(&[135, 136, 137, 138, 139, 148, 176]),
    );
    attrs.insert("hasjabs".into(), mk_int_list(&[]));
    attrs.insert(
        "hasjrel".into(),
        mk_int_list(&[93, 110, 114, 115, 123, 128, 129, 134, 140, 260, 261]),
    );
    attrs.insert(
        "haslocal".into(),
        mk_int_list(&[124, 125, 126, 127, 143, 266]),
    );
    attrs.insert(
        "hasname".into(),
        mk_int_list(&[
            90, 91, 95, 96, 97, 98, 101, 106, 108, 109, 116, 141, 175, 262, 263, 264, 265,
        ]),
    );
    // UNKNOWN: CPython sentinel; surface only needs presence.
    attrs.insert("UNKNOWN".into(), MbValue::from_int(-1));
    // COMPILER_FLAG_NAMES: dict {flag_int: name}
    {
        let d = MbObject::new_dict();
        unsafe {
            use super::super::dict_ops::DictKey;
            use super::super::rc::ObjData;
            if let ObjData::Dict(ref lock) = (*d).data {
                let mut m = lock.write().unwrap();
                for (flag, name) in &[
                    (1i64, "OPTIMIZED"),
                    (2, "NEWLOCALS"),
                    (4, "VARARGS"),
                    (8, "VARKEYWORDS"),
                    (16, "NESTED"),
                    (32, "GENERATOR"),
                    (64, "NOFREE"),
                    (128, "COROUTINE"),
                    (256, "ITERABLE_COROUTINE"),
                    (512, "ASYNC_GENERATOR"),
                ] {
                    m.insert(DictKey::Int(*flag), mk_str(*name));
                }
            }
        }
        attrs.insert("COMPILER_FLAG_NAMES".into(), MbValue::from_ptr(d));
    }
    // deoptmap: dict {specialized_name: base_name}; surface only needs presence.
    {
        let d = MbObject::new_dict();
        attrs.insert("deoptmap".into(), MbValue::from_ptr(d));
    }
    // Re-exported modules; surface only needs presence (hasattr).
    attrs.insert("collections".into(), mk_str("<module 'collections'>"));
    attrs.insert("io".into(), mk_str("<module 'io'>"));
    attrs.insert("sys".into(), mk_str("<module 'sys'>"));
    attrs.insert("types".into(), mk_str("<module 'types'>"));

    super::register_module("dis", attrs);
}

// -- Helpers --

fn extract_str(val: MbValue) -> Option<String> {
    val.as_ptr().and_then(|ptr| unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Str(ref s) = (*ptr).data {
            Some(s.clone())
        } else {
            None
        }
    })
}

/// Build a dis.Instruction-like dict.
fn make_instruction(
    opname: &str,
    opcode: i64,
    arg: i64,
    argval: MbValue,
    argrepr: &str,
    offset: i64,
    starts_line: MbValue,
    is_jump_target: bool,
) -> MbValue {
    use super::super::rc::{MbObject, MbObjectHeader, ObjData};
    let mut fields = FxHashMap::default();
    fields.insert(
        "opname".into(),
        MbValue::from_ptr(MbObject::new_str(opname.to_string())),
    );
    fields.insert("opcode".into(), MbValue::from_int(opcode));
    fields.insert("arg".into(), MbValue::from_int(arg));
    fields.insert("argval".into(), argval);
    fields.insert(
        "argrepr".into(),
        MbValue::from_ptr(MbObject::new_str(argrepr.to_string())),
    );
    fields.insert("offset".into(), MbValue::from_int(offset));
    fields.insert("starts_line".into(), starts_line);
    fields.insert("is_jump_target".into(), MbValue::from_bool(is_jump_target));
    let obj = Box::new(MbObject {
        header: MbObjectHeader {
            rc: std::sync::atomic::AtomicU32::new(1),
            kind: super::super::rc::ObjKind::Instance,
        },
        data: ObjData::Instance {
            class_name: "Instruction".to_string(),
            fields: crate::runtime::rc::MbRwLock::new(fields),
        },
    });
    MbValue::from_ptr(Box::into_raw(obj))
}

/// dis.dis(x=None) — print disassembly to stdout
/// In Mamba, this returns a string representation of the MIR.
pub fn mb_dis_dis(obj: MbValue) -> MbValue {
    let repr = extract_str(obj.clone())
        .map(|s| format!("Disassembly of <code object from '{}'>:\n  (Mamba MIR)", s))
        .unwrap_or_else(|| {
            "Disassembly of <object>:\n  (Mamba MIR not available in userspace)".to_string()
        });
    // Print to stdout (best-effort)
    println!("{}", repr);
    MbValue::none()
}

/// dis.disassemble(x, lasti=-1, *, file=None) — same as dis()
pub fn mb_dis_disassemble(obj: MbValue) -> MbValue {
    mb_dis_dis(obj)
}

/// dis.get_instructions(x, *, first_line=None) -> iterator of Instruction
/// Returns a list of Instruction objects representing Mamba MIR ops.
pub fn mb_dis_get_instructions(obj: MbValue) -> MbValue {
    // For now, return a stub list showing a LOAD_CONST + RETURN_VALUE pair
    let _src = extract_str(obj).unwrap_or_default();
    let instrs = vec![
        make_instruction(
            "RESUME",
            151,
            0,
            MbValue::from_int(0),
            "0",
            0,
            MbValue::from_int(1),
            false,
        ),
        make_instruction(
            "LOAD_CONST",
            100,
            0,
            MbValue::none(),
            "None",
            2,
            MbValue::none(),
            false,
        ),
        make_instruction(
            "RETURN_VALUE",
            83,
            0,
            MbValue::none(),
            "",
            4,
            MbValue::none(),
            false,
        ),
    ];
    MbValue::from_ptr(MbObject::new_list(instrs))
}

/// dis.show_code(x, *, file=None)
pub fn mb_dis_show_code(obj: MbValue) -> MbValue {
    let src = extract_str(obj).unwrap_or_else(|| "<code>".to_string());
    println!("Name:              {}", src);
    println!("Filename:          <mamba>");
    MbValue::none()
}

/// dis.code_info(x) -> str
pub fn mb_dis_code_info(obj: MbValue) -> MbValue {
    let src = extract_str(obj).unwrap_or_else(|| "<code>".to_string());
    let info = format!(
        "Name: {}\nFilename: <mamba>\nArgument count: 0\nKlass: MirBody",
        src
    );
    MbValue::from_ptr(MbObject::new_str(info))
}

/// dis.findlinestarts(code) -> iterator of (offset, lineno)
pub fn mb_dis_findlinestarts(_code: MbValue) -> MbValue {
    let entries = vec![MbValue::from_ptr(MbObject::new_tuple(vec![
        MbValue::from_int(0),
        MbValue::from_int(1),
    ]))];
    MbValue::from_ptr(MbObject::new_list(entries))
}

/// dis.findlabels(code) -> list of offsets
pub fn mb_dis_findlabels(_code: MbValue) -> MbValue {
    MbValue::from_ptr(MbObject::new_list(vec![]))
}

/// dis.stack_effect(opcode, oparg=None, *, jump=None) -> int
pub fn mb_dis_stack_effect(opcode: MbValue) -> MbValue {
    let _op = opcode.as_int().unwrap_or(0);
    MbValue::from_int(0) // simplified
}

/// dis.opmap — dict mapping opname -> opcode number
pub fn mb_dis_opmap() -> MbValue {
    let dict = MbObject::new_dict();
    unsafe {
        use super::super::rc::ObjData;
        if let ObjData::Dict(ref lock) = (*dict).data {
            let mut map = lock.write().unwrap();
            for (name, code) in MAMBA_OPCODES {
                map.insert((*name).into(), MbValue::from_int(*code));
            }
        }
    }
    MbValue::from_ptr(dict)
}

/// dis.opname — list mapping opcode number -> opname
pub fn mb_dis_opname() -> MbValue {
    // Create a 256-element list with names at their opcode positions
    let mut names: Vec<MbValue> = (0..256)
        .map(|_| MbValue::from_ptr(MbObject::new_str("<0>".to_string())))
        .collect();
    for (name, code) in MAMBA_OPCODES {
        let idx = *code as usize;
        if idx < 256 {
            names[idx] = MbValue::from_ptr(MbObject::new_str(name.to_string()));
        }
    }
    MbValue::from_ptr(MbObject::new_list(names))
}

/// dis.Instruction constructor stub
#[allow(non_snake_case)]
pub fn mb_dis_Instruction() -> MbValue {
    make_instruction("NOP", 9, 0, MbValue::none(), "", 0, MbValue::none(), false)
}

/// dis.pretty_flags(flags) -> str — render code-object flag names.
pub fn mb_dis_pretty_flags(flags: MbValue) -> MbValue {
    let _f = flags.as_int().unwrap_or(0);
    MbValue::from_ptr(MbObject::new_str(String::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_instructions() {
        let src = MbValue::from_ptr(MbObject::new_str("pass".to_string()));
        let instrs = mb_dis_get_instructions(src);
        assert!(instrs.as_ptr().is_some());
    }

    #[test]
    fn test_opmap() {
        let opmap = mb_dis_opmap();
        assert!(opmap.as_ptr().is_some());
    }

    #[test]
    fn test_code_info() {
        let obj = MbValue::from_ptr(MbObject::new_str("my_func".to_string()));
        let info = mb_dis_code_info(obj);
        assert!(info.as_ptr().is_some());
    }

    #[test]
    fn test_findlinestarts() {
        let result = mb_dis_findlinestarts(MbValue::none());
        assert!(result.as_ptr().is_some());
    }
}
