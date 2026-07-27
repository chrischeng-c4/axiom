use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/modulefinder/AddPackagePath__packagename_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_AddPackagePath__packagename_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "AddPackagePath__packagename_as_str_wrong"
# subject = "modulefinder.AddPackagePath(packagename: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.AddPackagePath(packagename: str); call it with the wrong type.

typeshed contract: packagename is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import AddPackagePath
try:
    AddPackagePath(12345, "")  # packagename: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__add_module__fqname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__add_module__fqname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__add_module__fqname_as_str_wrong"
# subject = "modulefinder.ModuleFinder.add_module(fqname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.add_module(fqname: str); call it with the wrong type.

typeshed contract: fqname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.add_module(12345)  # fqname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__determine_parent__caller_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__determine_parent__caller_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__determine_parent__caller_as_typed_wrong"
# subject = "modulefinder.ModuleFinder.determine_parent(caller: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.determine_parent(caller: typed); call it with the wrong type.

typeshed contract: caller is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.determine_parent(_W())  # caller: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__ensure_fromlist__m_as_Module_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__ensure_fromlist__m_as_Module_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__ensure_fromlist__m_as_Module_wrong"
# subject = "modulefinder.ModuleFinder.ensure_fromlist(m: Module)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.ensure_fromlist(m: Module); call it with the wrong type.

typeshed contract: m is Module. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.ensure_fromlist(_W(), None)  # m: Module <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__find_all_submodules__m_as_Module_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__find_all_submodules__m_as_Module_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__find_all_submodules__m_as_Module_wrong"
# subject = "modulefinder.ModuleFinder.find_all_submodules(m: Module)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.find_all_submodules(m: Module); call it with the wrong type.

typeshed contract: m is Module. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.find_all_submodules(_W())  # m: Module <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__find_head_package__parent_as_Module_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__find_head_package__parent_as_Module_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__find_head_package__parent_as_Module_wrong"
# subject = "modulefinder.ModuleFinder.find_head_package(parent: Module)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.find_head_package(parent: Module); call it with the wrong type.

typeshed contract: parent is Module. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.find_head_package(_W(), "")  # parent: Module <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__find_module__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__find_module__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__find_module__name_as_str_wrong"
# subject = "modulefinder.ModuleFinder.find_module(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.find_module(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.find_module(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__import_hook__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__import_hook__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__import_hook__name_as_str_wrong"
# subject = "modulefinder.ModuleFinder.import_hook(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.import_hook(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.import_hook(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__import_module__partname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__import_module__partname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__import_module__partname_as_str_wrong"
# subject = "modulefinder.ModuleFinder.import_module(partname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.import_module(partname: str); call it with the wrong type.

typeshed contract: partname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.import_module(12345, "", None)  # partname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__init__path_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__init__path_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__init__path_as_typed_wrong"
# subject = "modulefinder.ModuleFinder.__init__(path: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.__init__(path: typed); call it with the wrong type.

typeshed contract: path is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from modulefinder import ModuleFinder
try:
    ModuleFinder(_W())  # path: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__load_file__pathname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__load_file__pathname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__load_file__pathname_as_str_wrong"
# subject = "modulefinder.ModuleFinder.load_file(pathname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.load_file(pathname: str); call it with the wrong type.

typeshed contract: pathname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.load_file(12345)  # pathname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__load_module__fqname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__load_module__fqname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__load_module__fqname_as_str_wrong"
# subject = "modulefinder.ModuleFinder.load_module(fqname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.load_module(fqname: str); call it with the wrong type.

typeshed contract: fqname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.load_module(12345, None, "", None)  # fqname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__load_package__fqname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__load_package__fqname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__load_package__fqname_as_str_wrong"
# subject = "modulefinder.ModuleFinder.load_package(fqname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.load_package(fqname: str); call it with the wrong type.

typeshed contract: fqname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.load_package(12345, "")  # fqname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__load_tail__q_as_Module_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__load_tail__q_as_Module_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__load_tail__q_as_Module_wrong"
# subject = "modulefinder.ModuleFinder.load_tail(q: Module)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.load_tail(q: Module); call it with the wrong type.

typeshed contract: q is Module. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.load_tail(_W(), "")  # q: Module <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__replace_paths_in_code__co_as_CodeType_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__replace_paths_in_code__co_as_CodeType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__replace_paths_in_code__co_as_CodeType_wrong"
# subject = "modulefinder.ModuleFinder.replace_paths_in_code(co: CodeType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.replace_paths_in_code(co: CodeType); call it with the wrong type.

typeshed contract: co is CodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.replace_paths_in_code(_W())  # co: CodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__run_script__pathname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__run_script__pathname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__run_script__pathname_as_str_wrong"
# subject = "modulefinder.ModuleFinder.run_script(pathname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.run_script(pathname: str); call it with the wrong type.

typeshed contract: pathname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.run_script(12345)  # pathname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__scan_code__co_as_CodeType_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__scan_code__co_as_CodeType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__scan_code__co_as_CodeType_wrong"
# subject = "modulefinder.ModuleFinder.scan_code(co: CodeType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.scan_code(co: CodeType); call it with the wrong type.

typeshed contract: co is CodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.scan_code(_W(), None)  # co: CodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ModuleFinder__scan_opcodes__co_as_CodeType_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ModuleFinder__scan_opcodes__co_as_CodeType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__scan_opcodes__co_as_CodeType_wrong"
# subject = "modulefinder.ModuleFinder.scan_opcodes(co: CodeType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.scan_opcodes(co: CodeType); call it with the wrong type.

typeshed contract: co is CodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.scan_opcodes(_W())  # co: CodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/Module__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_Module__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "Module__init__name_as_str_wrong"
# subject = "modulefinder.Module.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.Module.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import Module
try:
    Module(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/modulefinder/ReplacePackage__oldname_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_modulefinder_ReplacePackage__oldname_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ReplacePackage__oldname_as_str_wrong"
# subject = "modulefinder.ReplacePackage(oldname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ReplacePackage(oldname: str); call it with the wrong type.

typeshed contract: oldname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from modulefinder import ReplacePackage
try:
    ReplacePackage(12345, "")  # oldname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
