use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/importlib_resources__functional/contents__anchor_as_Anchor_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources__functional_contents__anchor_as_Anchor_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources__functional"
# dimension = "type"
# case = "contents__anchor_as_Anchor_wrong"
# subject = "importlib.resources._functional.contents(anchor: Anchor)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/_functional.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources._functional.contents(anchor: Anchor); call it with the wrong type.

typeshed contract: anchor is Anchor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources._functional import contents
try:
    contents(_W())  # anchor: Anchor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_resources__functional/is_resource__anchor_as_Anchor_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources__functional_is_resource__anchor_as_Anchor_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources__functional"
# dimension = "type"
# case = "is_resource__anchor_as_Anchor_wrong"
# subject = "importlib.resources._functional.is_resource(anchor: Anchor)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/_functional.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources._functional.is_resource(anchor: Anchor); call it with the wrong type.

typeshed contract: anchor is Anchor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources._functional import is_resource
try:
    is_resource(_W())  # anchor: Anchor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_resources__functional/open_binary__anchor_as_Anchor_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources__functional_open_binary__anchor_as_Anchor_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources__functional"
# dimension = "type"
# case = "open_binary__anchor_as_Anchor_wrong"
# subject = "importlib.resources._functional.open_binary(anchor: Anchor)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/_functional.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources._functional.open_binary(anchor: Anchor); call it with the wrong type.

typeshed contract: anchor is Anchor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources._functional import open_binary
try:
    open_binary(_W())  # anchor: Anchor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_resources__functional/open_text__anchor_as_Anchor_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources__functional_open_text__anchor_as_Anchor_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources__functional"
# dimension = "type"
# case = "open_text__anchor_as_Anchor_wrong"
# subject = "importlib.resources._functional.open_text(anchor: Anchor)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/_functional.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources._functional.open_text(anchor: Anchor); call it with the wrong type.

typeshed contract: anchor is Anchor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources._functional import open_text
try:
    open_text(_W())  # anchor: Anchor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_resources__functional/path__anchor_as_Anchor_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources__functional_path__anchor_as_Anchor_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources__functional"
# dimension = "type"
# case = "path__anchor_as_Anchor_wrong"
# subject = "importlib.resources._functional.path(anchor: Anchor)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/_functional.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources._functional.path(anchor: Anchor); call it with the wrong type.

typeshed contract: anchor is Anchor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources._functional import path
try:
    path(_W())  # anchor: Anchor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_resources__functional/read_binary__anchor_as_Anchor_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources__functional_read_binary__anchor_as_Anchor_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources__functional"
# dimension = "type"
# case = "read_binary__anchor_as_Anchor_wrong"
# subject = "importlib.resources._functional.read_binary(anchor: Anchor)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/_functional.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources._functional.read_binary(anchor: Anchor); call it with the wrong type.

typeshed contract: anchor is Anchor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources._functional import read_binary
try:
    read_binary(_W())  # anchor: Anchor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/importlib_resources__functional/read_text__anchor_as_Anchor_wrong.py`.
#[test]
fn test_gen_type_std_libs_importlib_resources__functional_read_text__anchor_as_Anchor_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_resources__functional"
# dimension = "type"
# case = "read_text__anchor_as_Anchor_wrong"
# subject = "importlib.resources._functional.read_text(anchor: Anchor)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/resources/_functional.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.resources._functional.read_text(anchor: Anchor); call it with the wrong type.

typeshed contract: anchor is Anchor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.resources._functional import read_text
try:
    read_text(_W())  # anchor: Anchor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
