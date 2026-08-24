use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/BuiltinImporter__create_module__spec_as_ModuleSpec_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_BuiltinImporter__create_module__spec_as_ModuleSpec_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "BuiltinImporter__create_module__spec_as_ModuleSpec_wrong"
# subject = "_frozen_importlib.BuiltinImporter.create_module(spec: ModuleSpec)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.BuiltinImporter.create_module(spec: ModuleSpec); call it with the wrong type.

typeshed contract: spec is ModuleSpec. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _frozen_importlib import BuiltinImporter
try:
    BuiltinImporter.create_module(_W())  # spec: ModuleSpec <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/BuiltinImporter__exec_module__module_as_ModuleType_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_BuiltinImporter__exec_module__module_as_ModuleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "BuiltinImporter__exec_module__module_as_ModuleType_wrong"
# subject = "_frozen_importlib.BuiltinImporter.exec_module(module: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.BuiltinImporter.exec_module(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _frozen_importlib import BuiltinImporter
try:
    BuiltinImporter.exec_module(_W())  # module: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/BuiltinImporter__find_module__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_BuiltinImporter__find_module__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "BuiltinImporter__find_module__fullname_as_str_wrong"
# subject = "_frozen_importlib.BuiltinImporter.find_module(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.BuiltinImporter.find_module(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import BuiltinImporter
try:
    BuiltinImporter.find_module(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/BuiltinImporter__find_spec__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_BuiltinImporter__find_spec__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "BuiltinImporter__find_spec__fullname_as_str_wrong"
# subject = "_frozen_importlib.BuiltinImporter.find_spec(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.BuiltinImporter.find_spec(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import BuiltinImporter
try:
    BuiltinImporter.find_spec(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/BuiltinImporter__get_code__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_BuiltinImporter__get_code__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "BuiltinImporter__get_code__fullname_as_str_wrong"
# subject = "_frozen_importlib.BuiltinImporter.get_code(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.BuiltinImporter.get_code(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import BuiltinImporter
try:
    BuiltinImporter.get_code(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/BuiltinImporter__get_source__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_BuiltinImporter__get_source__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "BuiltinImporter__get_source__fullname_as_str_wrong"
# subject = "_frozen_importlib.BuiltinImporter.get_source(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.BuiltinImporter.get_source(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import BuiltinImporter
try:
    BuiltinImporter.get_source(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/BuiltinImporter__is_package__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_BuiltinImporter__is_package__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "BuiltinImporter__is_package__fullname_as_str_wrong"
# subject = "_frozen_importlib.BuiltinImporter.is_package(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.BuiltinImporter.is_package(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import BuiltinImporter
try:
    BuiltinImporter.is_package(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/BuiltinImporter__load_module__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_BuiltinImporter__load_module__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "BuiltinImporter__load_module__fullname_as_str_wrong"
# subject = "_frozen_importlib.BuiltinImporter.load_module(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.BuiltinImporter.load_module(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import BuiltinImporter
try:
    BuiltinImporter.load_module(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/BuiltinImporter__module_repr__module_as_ModuleType_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_BuiltinImporter__module_repr__module_as_ModuleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "BuiltinImporter__module_repr__module_as_ModuleType_wrong"
# subject = "_frozen_importlib.BuiltinImporter.module_repr(module: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.BuiltinImporter.module_repr(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _frozen_importlib import BuiltinImporter
try:
    BuiltinImporter.module_repr(_W())  # module: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/FrozenImporter__create_module__spec_as_ModuleSpec_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_FrozenImporter__create_module__spec_as_ModuleSpec_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "FrozenImporter__create_module__spec_as_ModuleSpec_wrong"
# subject = "_frozen_importlib.FrozenImporter.create_module(spec: ModuleSpec)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.FrozenImporter.create_module(spec: ModuleSpec); call it with the wrong type.

typeshed contract: spec is ModuleSpec. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _frozen_importlib import FrozenImporter
try:
    FrozenImporter.create_module(_W())  # spec: ModuleSpec <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/FrozenImporter__exec_module__module_as_ModuleType_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_FrozenImporter__exec_module__module_as_ModuleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "FrozenImporter__exec_module__module_as_ModuleType_wrong"
# subject = "_frozen_importlib.FrozenImporter.exec_module(module: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.FrozenImporter.exec_module(module: ModuleType); call it with the wrong type.

typeshed contract: module is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _frozen_importlib import FrozenImporter
try:
    FrozenImporter.exec_module(_W())  # module: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/FrozenImporter__find_module__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_FrozenImporter__find_module__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "FrozenImporter__find_module__fullname_as_str_wrong"
# subject = "_frozen_importlib.FrozenImporter.find_module(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.FrozenImporter.find_module(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import FrozenImporter
try:
    FrozenImporter.find_module(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/FrozenImporter__find_spec__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_FrozenImporter__find_spec__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "FrozenImporter__find_spec__fullname_as_str_wrong"
# subject = "_frozen_importlib.FrozenImporter.find_spec(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.FrozenImporter.find_spec(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import FrozenImporter
try:
    FrozenImporter.find_spec(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/FrozenImporter__get_code__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_FrozenImporter__get_code__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "FrozenImporter__get_code__fullname_as_str_wrong"
# subject = "_frozen_importlib.FrozenImporter.get_code(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.FrozenImporter.get_code(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import FrozenImporter
try:
    FrozenImporter.get_code(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/FrozenImporter__get_source__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_FrozenImporter__get_source__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "FrozenImporter__get_source__fullname_as_str_wrong"
# subject = "_frozen_importlib.FrozenImporter.get_source(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.FrozenImporter.get_source(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import FrozenImporter
try:
    FrozenImporter.get_source(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/FrozenImporter__is_package__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_FrozenImporter__is_package__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "FrozenImporter__is_package__fullname_as_str_wrong"
# subject = "_frozen_importlib.FrozenImporter.is_package(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.FrozenImporter.is_package(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import FrozenImporter
try:
    FrozenImporter.is_package(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/FrozenImporter__load_module__fullname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_FrozenImporter__load_module__fullname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "FrozenImporter__load_module__fullname_as_str_wrong"
# subject = "_frozen_importlib.FrozenImporter.load_module(fullname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.FrozenImporter.load_module(fullname: str); call it with the wrong type.

typeshed contract: fullname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import FrozenImporter
try:
    FrozenImporter.load_module(12345)  # fullname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/FrozenImporter__module_repr__m_as_ModuleType_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_FrozenImporter__module_repr__m_as_ModuleType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "FrozenImporter__module_repr__m_as_ModuleType_wrong"
# subject = "_frozen_importlib.FrozenImporter.module_repr(m: ModuleType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.FrozenImporter.module_repr(m: ModuleType); call it with the wrong type.

typeshed contract: m is ModuleType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _frozen_importlib import FrozenImporter
try:
    FrozenImporter.module_repr(_W())  # m: ModuleType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/ModuleSpec__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_ModuleSpec__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "ModuleSpec__init__name_as_str_wrong"
# subject = "_frozen_importlib.ModuleSpec.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.ModuleSpec.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import ModuleSpec
try:
    ModuleSpec(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/module_from_spec__spec_as_ModuleSpec_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_module_from_spec__spec_as_ModuleSpec_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "module_from_spec__spec_as_ModuleSpec_wrong"
# subject = "_frozen_importlib.module_from_spec(spec: ModuleSpec)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.module_from_spec(spec: ModuleSpec); call it with the wrong type.

typeshed contract: spec is ModuleSpec. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _frozen_importlib import module_from_spec
try:
    module_from_spec(_W())  # spec: ModuleSpec <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_frozen_importlib/spec_from_loader__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__frozen_importlib_spec_from_loader__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib"
# dimension = "type"
# case = "spec_from_loader__name_as_str_wrong"
# subject = "_frozen_importlib.spec_from_loader(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib.spec_from_loader(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _frozen_importlib import spec_from_loader
try:
    spec_from_loader(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
