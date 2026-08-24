use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/faulthandler/dump_c_stack__file_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_faulthandler_dump_c_stack__file_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "type"
# case = "dump_c_stack__file_as_FileDescriptorLike_wrong"
# subject = "faulthandler.dump_c_stack(file: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/faulthandler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: faulthandler.dump_c_stack(file: FileDescriptorLike); call it with the wrong type.

typeshed contract: file is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from faulthandler import dump_c_stack
try:
    dump_c_stack(_W())  # file: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/faulthandler/dump_traceback__file_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_faulthandler_dump_traceback__file_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "type"
# case = "dump_traceback__file_as_FileDescriptorLike_wrong"
# subject = "faulthandler.dump_traceback(file: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/faulthandler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: faulthandler.dump_traceback(file: FileDescriptorLike); call it with the wrong type.

typeshed contract: file is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from faulthandler import dump_traceback
try:
    dump_traceback(_W())  # file: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/faulthandler/dump_traceback_later__timeout_as_float_wrong.py`.
#[test]
fn test_gen_type_std_libs_faulthandler_dump_traceback_later__timeout_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "type"
# case = "dump_traceback_later__timeout_as_float_wrong"
# subject = "faulthandler.dump_traceback_later(timeout: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/faulthandler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: faulthandler.dump_traceback_later(timeout: float); call it with the wrong type.

typeshed contract: timeout is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from faulthandler import dump_traceback_later
try:
    dump_traceback_later("not_a_float")  # timeout: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/faulthandler/enable__file_as_FileDescriptorLike_wrong.py`.
#[test]
fn test_gen_type_std_libs_faulthandler_enable__file_as_FileDescriptorLike_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "type"
# case = "enable__file_as_FileDescriptorLike_wrong"
# subject = "faulthandler.enable(file: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/faulthandler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: faulthandler.enable(file: FileDescriptorLike); call it with the wrong type.

typeshed contract: file is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from faulthandler import enable
try:
    enable(_W())  # file: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/faulthandler/register__signum_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_faulthandler_register__signum_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "type"
# case = "register__signum_as_int_wrong"
# subject = "faulthandler.register(signum: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/faulthandler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: faulthandler.register(signum: int); call it with the wrong type.

typeshed contract: signum is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from faulthandler import register
try:
    register("not_an_int")  # signum: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/faulthandler/unregister__signum_as_int_wrong.py`.
#[test]
fn test_gen_type_std_libs_faulthandler_unregister__signum_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "type"
# case = "unregister__signum_as_int_wrong"
# subject = "faulthandler.unregister(signum: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/faulthandler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: faulthandler.unregister(signum: int); call it with the wrong type.

typeshed contract: signum is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from faulthandler import unregister
try:
    unregister("not_an_int")  # signum: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
