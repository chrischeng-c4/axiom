use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/zipapp/create_archive__source_as__Path_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipapp_create_archive__source_as__Path_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipapp"
# dimension = "type"
# case = "create_archive__source_as__Path_wrong"
# subject = "zipapp.create_archive(source: _Path)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipapp.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipapp.create_archive(source: _Path); call it with the wrong type.

typeshed contract: source is _Path. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipapp import create_archive
try:
    create_archive(_W())  # source: _Path <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/zipapp/get_interpreter__archive_as__Path_wrong.py`.
#[test]
fn test_gen_type_std_libs_zipapp_get_interpreter__archive_as__Path_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipapp"
# dimension = "type"
# case = "get_interpreter__archive_as__Path_wrong"
# subject = "zipapp.get_interpreter(archive: _Path)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipapp.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipapp.get_interpreter(archive: _Path); call it with the wrong type.

typeshed contract: archive is _Path. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipapp import get_interpreter
try:
    get_interpreter(_W())  # archive: _Path <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
