use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/dis/Bytecode__from_traceback__tb_as_TracebackType_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_Bytecode__from_traceback__tb_as_TracebackType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "Bytecode__from_traceback__tb_as_TracebackType_wrong"
# subject = "dis.Bytecode.from_traceback(tb: TracebackType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.Bytecode.from_traceback(tb: TracebackType); call it with the wrong type.

typeshed contract: tb is TracebackType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import Bytecode
try:
    Bytecode.from_traceback(_W())  # tb: TracebackType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dis/Bytecode__init__x_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_Bytecode__init__x_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "Bytecode__init__x_as_typed_wrong"
# subject = "dis.Bytecode.__init__(x: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.Bytecode.__init__(x: typed); call it with the wrong type.

typeshed contract: x is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import Bytecode
try:
    Bytecode(_W())  # x: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dis/Instruction__make__opname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_Instruction__make__opname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "Instruction__make__opname_as_str_wrong"
# subject = "dis.Instruction.make(opname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.Instruction.make(opname: str); call it with the wrong type.

typeshed contract: opname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from dis import Instruction
try:
    Instruction.make(12345, None, None, "", 0, 0, True, None)  # opname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dis/code_info__x_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_code_info__x_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "code_info__x_as_typed_wrong"
# subject = "dis.code_info(x: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.code_info(x: typed); call it with the wrong type.

typeshed contract: x is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import code_info
try:
    code_info(_W())  # x: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dis/dis__x_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_dis__x_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "dis__x_as_typed_wrong"
# subject = "dis.dis(x: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.dis(x: typed); call it with the wrong type.

typeshed contract: x is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import dis
try:
    dis(_W())  # x: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dis/disassemble__co_as__HaveCodeType_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_disassemble__co_as__HaveCodeType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "disassemble__co_as__HaveCodeType_wrong"
# subject = "dis.disassemble(co: _HaveCodeType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.disassemble(co: _HaveCodeType); call it with the wrong type.

typeshed contract: co is _HaveCodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import disassemble
try:
    disassemble(_W())  # co: _HaveCodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dis/distb__tb_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_distb__tb_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "distb__tb_as_typed_wrong"
# subject = "dis.distb(tb: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.distb(tb: typed); call it with the wrong type.

typeshed contract: tb is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import distb
try:
    distb(_W())  # tb: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dis/findlabels__code_as__HaveCodeType_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_findlabels__code_as__HaveCodeType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "findlabels__code_as__HaveCodeType_wrong"
# subject = "dis.findlabels(code: _HaveCodeType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.findlabels(code: _HaveCodeType); call it with the wrong type.

typeshed contract: code is _HaveCodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import findlabels
try:
    findlabels(_W())  # code: _HaveCodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dis/findlinestarts__code_as__HaveCodeType_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_findlinestarts__code_as__HaveCodeType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "findlinestarts__code_as__HaveCodeType_wrong"
# subject = "dis.findlinestarts(code: _HaveCodeType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.findlinestarts(code: _HaveCodeType); call it with the wrong type.

typeshed contract: code is _HaveCodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import findlinestarts
try:
    findlinestarts(_W())  # code: _HaveCodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dis/get_instructions__x_as__HaveCodeType_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_get_instructions__x_as__HaveCodeType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "get_instructions__x_as__HaveCodeType_wrong"
# subject = "dis.get_instructions(x: _HaveCodeType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.get_instructions(x: _HaveCodeType); call it with the wrong type.

typeshed contract: x is _HaveCodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import get_instructions
try:
    get_instructions(_W())  # x: _HaveCodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dis/pretty_flags__flags_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_pretty_flags__flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "pretty_flags__flags_as_int_wrong"
# subject = "dis.pretty_flags(flags: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.pretty_flags(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from dis import pretty_flags
try:
    pretty_flags("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/dis/show_code__co_as__HaveCodeType_wrong.py`.
#[test]
fn test_gen_type_std_libs_dis_show_code__co_as__HaveCodeType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dis"
# dimension = "type"
# case = "show_code__co_as__HaveCodeType_wrong"
# subject = "dis.show_code(co: _HaveCodeType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/dis.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: dis.show_code(co: _HaveCodeType); call it with the wrong type.

typeshed contract: co is _HaveCodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from dis import show_code
try:
    show_code(_W())  # co: _HaveCodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
