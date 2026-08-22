use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/_warnings/warn__message_as_Warning_wrong.py`.
#[test]
fn test_gen_type_std_libs__warnings_warn__message_as_Warning_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_warnings"
# dimension = "type"
# case = "warn__message_as_Warning_wrong"
# subject = "_warnings.warn(message: Warning)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _warnings.warn(message: Warning); call it with the wrong type.

typeshed contract: message is Warning. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _warnings import warn
try:
    warn(_W())  # message: Warning <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_warnings/warn__message_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__warnings_warn__message_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_warnings"
# dimension = "type"
# case = "warn__message_as_str_wrong"
# subject = "_warnings.warn(message: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _warnings.warn(message: str); call it with the wrong type.

typeshed contract: message is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _warnings import warn
try:
    warn(12345)  # message: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_warnings/warn_explicit__message_as_Warning_wrong.py`.
#[test]
fn test_gen_type_std_libs__warnings_warn_explicit__message_as_Warning_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_warnings"
# dimension = "type"
# case = "warn_explicit__message_as_Warning_wrong"
# subject = "_warnings.warn_explicit(message: Warning)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _warnings.warn_explicit(message: Warning); call it with the wrong type.

typeshed contract: message is Warning. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _warnings import warn_explicit
try:
    warn_explicit(_W(), None, "", 0)  # message: Warning <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/_warnings/warn_explicit__message_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs__warnings_warn_explicit__message_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_warnings"
# dimension = "type"
# case = "warn_explicit__message_as_str_wrong"
# subject = "_warnings.warn_explicit(message: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _warnings.warn_explicit(message: str); call it with the wrong type.

typeshed contract: message is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _warnings import warn_explicit
try:
    warn_explicit(12345, None, "", 0)  # message: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/warnings/WarningMessage__init__message_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_warnings_WarningMessage__init__message_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "type"
# case = "WarningMessage__init__message_as_typed_wrong"
# subject = "warnings.WarningMessage.__init__(message: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: warnings.WarningMessage.__init__(message: typed); call it with the wrong type.

typeshed contract: message is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from warnings import WarningMessage
try:
    WarningMessage(_W(), None, "", 0)  # message: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/warnings/deprecated__init__message_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_std_libs_warnings_deprecated__init__message_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "type"
# case = "deprecated__init__message_as_LiteralString_wrong"
# subject = "warnings.deprecated.__init__(message: LiteralString)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: warnings.deprecated.__init__(message: LiteralString); call it with the wrong type.

typeshed contract: message is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from warnings import deprecated
try:
    deprecated(_W())  # message: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/warnings/filterwarnings__action_as__ActionKind_wrong.py`.
#[test]
fn test_gen_type_std_libs_warnings_filterwarnings__action_as__ActionKind_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "type"
# case = "filterwarnings__action_as__ActionKind_wrong"
# subject = "warnings.filterwarnings(action: _ActionKind)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: warnings.filterwarnings(action: _ActionKind); call it with the wrong type.

typeshed contract: action is _ActionKind. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from warnings import filterwarnings
try:
    filterwarnings(_W())  # action: _ActionKind <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/warnings/formatwarning__message_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_warnings_formatwarning__message_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "type"
# case = "formatwarning__message_as_typed_wrong"
# subject = "warnings.formatwarning(message: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: warnings.formatwarning(message: typed); call it with the wrong type.

typeshed contract: message is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from warnings import formatwarning
try:
    formatwarning(_W(), None, "", 0)  # message: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/warnings/showwarning__message_as_typed_wrong.py`.
#[test]
fn test_gen_type_std_libs_warnings_showwarning__message_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "type"
# case = "showwarning__message_as_typed_wrong"
# subject = "warnings.showwarning(message: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: warnings.showwarning(message: typed); call it with the wrong type.

typeshed contract: message is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from warnings import showwarning
try:
    showwarning(_W(), None, "", 0)  # message: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/warnings/simplefilter__action_as__ActionKind_wrong.py`.
#[test]
fn test_gen_type_std_libs_warnings_simplefilter__action_as__ActionKind_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "type"
# case = "simplefilter__action_as__ActionKind_wrong"
# subject = "warnings.simplefilter(action: _ActionKind)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: warnings.simplefilter(action: _ActionKind); call it with the wrong type.

typeshed contract: action is _ActionKind. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from warnings import simplefilter
try:
    simplefilter(_W())  # action: _ActionKind <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
