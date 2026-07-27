use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseExceptionGroup____new____message_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseExceptionGroup____new____message_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseExceptionGroup____new____message_as_str_wrong"
# subject = "builtins.BaseExceptionGroup.__new__(message: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseExceptionGroup.__new__(message: str); call it with the wrong type.

typeshed contract: message is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import BaseExceptionGroup
try:
    BaseExceptionGroup.__new__(
        BaseExceptionGroup, 12345, [ValueError("x")]
    )  # message: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseExceptionGroup__derive__excs_as_Sequence_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseExceptionGroup__derive__excs_as_Sequence_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseExceptionGroup__derive__excs_as_Sequence_wrong"
# subject = "builtins.BaseExceptionGroup.derive(excs: Sequence)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseExceptionGroup.derive(excs: Sequence); call it with the wrong type.

typeshed contract: excs is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import BaseExceptionGroup
obj = BaseExceptionGroup("msg", [ValueError("x")])
try:
    obj.derive(_W())  # excs: Sequence <- wrong-typed
    print("no_typeerror:")  # mamba must reject the wrong-typed arg
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseExceptionGroup__init__message_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseExceptionGroup__init__message_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseExceptionGroup__init__message_as_str_wrong"
# subject = "builtins.BaseExceptionGroup.__init__(message: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseExceptionGroup.__init__(message: str); call it with the wrong type.

typeshed contract: message is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import BaseExceptionGroup
try:
    BaseExceptionGroup(12345, None)  # message: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseExceptionGroup__split__matcher_value_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseExceptionGroup__split__matcher_value_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseExceptionGroup__split__matcher_value_as_Callable_wrong"
# subject = "builtins.BaseExceptionGroup.split(matcher_value: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseExceptionGroup.split(matcher_value: Callable); call it with the wrong type.

typeshed contract: matcher_value is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import BaseExceptionGroup
obj = BaseExceptionGroup("msg", [ValueError("x")])
try:
    obj.split(_W())  # matcher_value: Callable <- wrong-typed
    print("no_typeerror:")  # mamba must reject the wrong-typed arg
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseExceptionGroup__split__matcher_value_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseExceptionGroup__split__matcher_value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseExceptionGroup__split__matcher_value_as_typed_wrong"
# subject = "builtins.BaseExceptionGroup.split(matcher_value: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseExceptionGroup.split(matcher_value: typed); call it with the wrong type.

typeshed contract: matcher_value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import BaseExceptionGroup
obj = BaseExceptionGroup("msg", [ValueError("x")])
try:
    obj.split(_W())  # matcher_value: typed <- wrong-typed
    print("no_typeerror:")  # mamba must reject the wrong-typed arg
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseExceptionGroup__subgroup__matcher_value_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseExceptionGroup__subgroup__matcher_value_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseExceptionGroup__subgroup__matcher_value_as_Callable_wrong"
# subject = "builtins.BaseExceptionGroup.subgroup(matcher_value: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseExceptionGroup.subgroup(matcher_value: Callable); call it with the wrong type.

typeshed contract: matcher_value is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import BaseExceptionGroup
obj = BaseExceptionGroup("msg", [ValueError("x")])
try:
    obj.subgroup(_W())  # matcher_value: Callable <- wrong-typed
    print("no_typeerror:")  # mamba must reject the wrong-typed arg
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseExceptionGroup__subgroup__matcher_value_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseExceptionGroup__subgroup__matcher_value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseExceptionGroup__subgroup__matcher_value_as_typed_wrong"
# subject = "builtins.BaseExceptionGroup.subgroup(matcher_value: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseExceptionGroup.subgroup(matcher_value: typed); call it with the wrong type.

typeshed contract: matcher_value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import BaseExceptionGroup
obj = BaseExceptionGroup("msg", [ValueError("x")])
try:
    obj.subgroup(_W())  # matcher_value: typed <- wrong-typed
    print("no_typeerror:")  # mamba must reject the wrong-typed arg
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseException____setstate____state_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseException____setstate____state_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseException____setstate____state_as_typed_wrong"
# subject = "builtins.BaseException.__setstate__(state: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseException.__setstate__(state: typed); call it with the wrong type.

typeshed contract: state is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import BaseException
obj = BaseException()
try:
    obj.__setstate__(_W())  # state: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseException__add_note__note_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseException__add_note__note_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseException__add_note__note_as_str_wrong"
# subject = "builtins.BaseException.add_note(note: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseException.add_note(note: str); call it with the wrong type.

typeshed contract: note is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import BaseException
obj = BaseException()
try:
    obj.add_note(12345)  # note: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseException__add_note__note_as_str_wrong_via_inferred_receiver.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseException__add_note__note_as_str_wrong_via_inferred_receiver() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseException__add_note__note_as_str_wrong_via_inferred_receiver"
# subject = "builtins.BaseException.add_note(note: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseException.add_note(note: str); call it with the wrong type.

Receiver-resolution variant (#886): `BaseException` is a global builtin, so
`e = BaseException(...)` carries no `from builtins import BaseException`
import statement -- the `instance_origins` provenance map (populated only by
a direct construction through an *imported* class qualifier) never sees `e`.
The Method wall must still fire by falling back to `e`'s already-inferred
`Ty::Class` name. A second hop (`e2 = e`) exercises the same fallback through
an indirect assignment chain.

typeshed contract: note is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise -- mamba's to
enforce)."""

e = BaseException("boom")
e2 = e
try:
    e2.add_note(12345)  # note: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as exc:
    print("typeerror:", type(exc).__name__)
except Exception as exc:
    print("setup_or_other:", type(exc).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseException__add_note__note_as_str_wrong_via_typed_param.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseException__add_note__note_as_str_wrong_via_typed_param() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseException__add_note__note_as_str_wrong_via_typed_param"
# subject = "builtins.BaseException.add_note(note: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseException.add_note(note: str); call it with the wrong type.

Receiver-resolution variant (#886): the receiver here is a function
*parameter* annotated `e: BaseException`, never assigned via a direct
`x = Cls(...)` construction that `instance_origins` tracks. The Method wall
must still fire by falling back to the parameter's inferred `Ty::Class` name
(from resolving the `BaseException` annotation), not the assignment-based
instance_origins map.

typeshed contract: note is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise -- mamba's to
enforce)."""


def annotate(e: BaseException):
    e.add_note(12345)  # note: str <- wrong-typed


try:
    annotate(BaseException("boom"))
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as exc:
    print("typeerror:", type(exc).__name__)
except Exception as exc:
    print("setup_or_other:", type(exc).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/BaseException__with_traceback__tb_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_BaseException__with_traceback__tb_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "BaseException__with_traceback__tb_as_typed_wrong"
# subject = "builtins.BaseException.with_traceback(tb: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.BaseException.with_traceback(tb: typed); call it with the wrong type.

typeshed contract: tb is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import BaseException
obj = BaseException()
try:
    obj.with_traceback(_W())  # tb: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/ExceptionGroup____new____message_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_ExceptionGroup____new____message_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "ExceptionGroup____new____message_as_str_wrong"
# subject = "builtins.ExceptionGroup.__new__(message: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.ExceptionGroup.__new__(message: str); call it with the wrong type.

typeshed contract: message is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import ExceptionGroup
try:
    ExceptionGroup.__new__(
        ExceptionGroup, 12345, [ValueError("x")]
    )  # message: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/ExceptionGroup__init__message_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_ExceptionGroup__init__message_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "ExceptionGroup__init__message_as_str_wrong"
# subject = "builtins.ExceptionGroup.__init__(message: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.ExceptionGroup.__init__(message: str); call it with the wrong type.

typeshed contract: message is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import ExceptionGroup
try:
    ExceptionGroup(12345, None)  # message: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/ExceptionGroup__split__matcher_value_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_ExceptionGroup__split__matcher_value_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "ExceptionGroup__split__matcher_value_as_Callable_wrong"
# subject = "builtins.ExceptionGroup.split(matcher_value: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.ExceptionGroup.split(matcher_value: Callable); call it with the wrong type.

typeshed contract: matcher_value is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import ExceptionGroup
obj = ExceptionGroup("msg", [ValueError("x")])
try:
    obj.split(_W())  # matcher_value: Callable <- wrong-typed
    print("no_typeerror:")  # mamba must reject the wrong-typed arg
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/ExceptionGroup__split__matcher_value_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_ExceptionGroup__split__matcher_value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "ExceptionGroup__split__matcher_value_as_typed_wrong"
# subject = "builtins.ExceptionGroup.split(matcher_value: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.ExceptionGroup.split(matcher_value: typed); call it with the wrong type.

typeshed contract: matcher_value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import ExceptionGroup
obj = ExceptionGroup("msg", [ValueError("x")])
try:
    obj.split(_W())  # matcher_value: typed <- wrong-typed
    print("no_typeerror:")  # mamba must reject the wrong-typed arg
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/ExceptionGroup__subgroup__matcher_value_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_ExceptionGroup__subgroup__matcher_value_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "ExceptionGroup__subgroup__matcher_value_as_Callable_wrong"
# subject = "builtins.ExceptionGroup.subgroup(matcher_value: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.ExceptionGroup.subgroup(matcher_value: Callable); call it with the wrong type.

typeshed contract: matcher_value is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import ExceptionGroup
obj = ExceptionGroup("msg", [ValueError("x")])
try:
    obj.subgroup(_W())  # matcher_value: Callable <- wrong-typed
    print("no_typeerror:")  # mamba must reject the wrong-typed arg
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/ExceptionGroup__subgroup__matcher_value_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_ExceptionGroup__subgroup__matcher_value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "ExceptionGroup__subgroup__matcher_value_as_typed_wrong"
# subject = "builtins.ExceptionGroup.subgroup(matcher_value: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.ExceptionGroup.subgroup(matcher_value: typed); call it with the wrong type.

typeshed contract: matcher_value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import ExceptionGroup
obj = ExceptionGroup("msg", [ValueError("x")])
try:
    obj.subgroup(_W())  # matcher_value: typed <- wrong-typed
    print("no_typeerror:")  # mamba must reject the wrong-typed arg
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/SyntaxError__init__msg_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_SyntaxError__init__msg_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "SyntaxError__init__msg_as_str_wrong"
# subject = "builtins.SyntaxError.__init__(msg: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.SyntaxError.__init__(msg: str); call it with the wrong type.

typeshed contract: msg is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import SyntaxError
try:
    SyntaxError(12345, None)  # msg: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/UnicodeDecodeError__init__encoding_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_UnicodeDecodeError__init__encoding_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "UnicodeDecodeError__init__encoding_as_str_wrong"
# subject = "builtins.UnicodeDecodeError.__init__(encoding: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.UnicodeDecodeError.__init__(encoding: str); call it with the wrong type.

typeshed contract: encoding is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import UnicodeDecodeError
try:
    UnicodeDecodeError(12345, None, 0, 0, "")  # encoding: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/UnicodeEncodeError__init__encoding_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_UnicodeEncodeError__init__encoding_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "UnicodeEncodeError__init__encoding_as_str_wrong"
# subject = "builtins.UnicodeEncodeError.__init__(encoding: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.UnicodeEncodeError.__init__(encoding: str); call it with the wrong type.

typeshed contract: encoding is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import UnicodeEncodeError
try:
    UnicodeEncodeError(12345, "", 0, 0, "")  # encoding: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/UnicodeTranslateError__init__object_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_UnicodeTranslateError__init__object_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "UnicodeTranslateError__init__object_as_str_wrong"
# subject = "builtins.UnicodeTranslateError.__init__(object: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.UnicodeTranslateError.__init__(object: str); call it with the wrong type.

typeshed contract: object is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import UnicodeTranslateError
try:
    UnicodeTranslateError(12345, 0, 0, "")  # object: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/abs__x_as_SupportsAbs_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_abs__x_as_SupportsAbs_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "abs__x_as_SupportsAbs_wrong"
# subject = "builtins.abs(x: SupportsAbs)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.abs(x: SupportsAbs); call it with the wrong type.

typeshed contract: x is SupportsAbs. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    abs(_W())  # x: SupportsAbs <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/aiter__async_iterable_as_SupportsAiter_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_aiter__async_iterable_as_SupportsAiter_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "aiter__async_iterable_as_SupportsAiter_wrong"
# subject = "builtins.aiter(async_iterable: SupportsAiter)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.aiter(async_iterable: SupportsAiter); call it with the wrong type.

typeshed contract: async_iterable is SupportsAiter. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    aiter(_W())  # async_iterable: SupportsAiter <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/all__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_all__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "all__iterable_as_Iterable_wrong"
# subject = "builtins.all(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.all(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    all(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/anext__i_as_SupportsAnext_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_anext__i_as_SupportsAnext_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "anext__i_as_SupportsAnext_wrong"
# subject = "builtins.anext(i: SupportsAnext)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.anext(i: SupportsAnext); call it with the wrong type.

typeshed contract: i is SupportsAnext. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    anext(_W(), None)  # i: SupportsAnext <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/anext__i_as__SupportsSynchronousAnext_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_anext__i_as__SupportsSynchronousAnext_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "anext__i_as__SupportsSynchronousAnext_wrong"
# subject = "builtins.anext(i: _SupportsSynchronousAnext)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.anext(i: _SupportsSynchronousAnext); call it with the wrong type.

typeshed contract: i is _SupportsSynchronousAnext. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    anext(_W())  # i: _SupportsSynchronousAnext <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/any__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_any__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "any__iterable_as_Iterable_wrong"
# subject = "builtins.any(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.any(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    any(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bin__number_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bin__number_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bin__number_as_SupportsIndex_wrong"
# subject = "builtins.bin(number: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bin(number: SupportsIndex); call it with the wrong type.

typeshed contract: number is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    bin(_W())  # number: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____and____value_as_bool_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____and____value_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____and____value_as_bool_wrong"
# subject = "builtins.bool.__and__(value: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__and__(value: bool); call it with the wrong type.

typeshed contract: value is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__and__("not_a_bool")  # value: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____and____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____and____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____and____value_as_int_wrong"
# subject = "builtins.bool.__and__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__and__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__and__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____or____value_as_bool_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____or____value_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____or____value_as_bool_wrong"
# subject = "builtins.bool.__or__(value: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__or__(value: bool); call it with the wrong type.

typeshed contract: value is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__or__("not_a_bool")  # value: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____or____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____or____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____or____value_as_int_wrong"
# subject = "builtins.bool.__or__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__or__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__or__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____rand____value_as_bool_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____rand____value_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____rand____value_as_bool_wrong"
# subject = "builtins.bool.__rand__(value: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__rand__(value: bool); call it with the wrong type.

typeshed contract: value is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__rand__("not_a_bool")  # value: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____rand____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____rand____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____rand____value_as_int_wrong"
# subject = "builtins.bool.__rand__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__rand__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__rand__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____ror____value_as_bool_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____ror____value_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____ror____value_as_bool_wrong"
# subject = "builtins.bool.__ror__(value: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__ror__(value: bool); call it with the wrong type.

typeshed contract: value is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__ror__("not_a_bool")  # value: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____ror____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____ror____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____ror____value_as_int_wrong"
# subject = "builtins.bool.__ror__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__ror__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__ror__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____rxor____value_as_bool_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____rxor____value_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____rxor____value_as_bool_wrong"
# subject = "builtins.bool.__rxor__(value: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__rxor__(value: bool); call it with the wrong type.

typeshed contract: value is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__rxor__("not_a_bool")  # value: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____rxor____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____rxor____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____rxor____value_as_int_wrong"
# subject = "builtins.bool.__rxor__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__rxor__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__rxor__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____xor____value_as_bool_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____xor____value_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____xor____value_as_bool_wrong"
# subject = "builtins.bool.__xor__(value: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__xor__(value: bool); call it with the wrong type.

typeshed contract: value is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__xor__("not_a_bool")  # value: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bool____xor____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bool____xor____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bool____xor____value_as_int_wrong"
# subject = "builtins.bool.__xor__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bool.__xor__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bool
obj = bool()
try:
    obj.__xor__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____add____value_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____add____value_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____add____value_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.__add__(value: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__add__(value: ReadableBuffer); call it with the wrong type.

typeshed contract: value is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__add__(_W())  # value: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____buffer____flags_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____buffer____flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____buffer____flags_as_int_wrong"
# subject = "builtins.bytearray.__buffer__(flags: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__buffer__(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__buffer__("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____contains____key_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____contains____key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____contains____key_as_typed_wrong"
# subject = "builtins.bytearray.__contains__(key: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__contains__(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__contains__(_W())  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____delitem____key_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____delitem____key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____delitem____key_as_typed_wrong"
# subject = "builtins.bytearray.__delitem__(key: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__delitem__(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray()
try:
    obj.__delitem__(_W())  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____ge____value_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____ge____value_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____ge____value_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.__ge__(value: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__ge__(value: ReadableBuffer); call it with the wrong type.

typeshed contract: value is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__ge__(_W())  # value: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____getitem____key_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____getitem____key_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____getitem____key_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.__getitem__(key: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__getitem__(key: SupportsIndex); call it with the wrong type.

typeshed contract: key is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray()
try:
    obj.__getitem__(_W())  # key: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____getitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____getitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____getitem____key_as_slice_wrong"
# subject = "builtins.bytearray.__getitem__(key: slice)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__getitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray()
try:
    obj.__getitem__(_W())  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____gt____value_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____gt____value_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____gt____value_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.__gt__(value: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__gt__(value: ReadableBuffer); call it with the wrong type.

typeshed contract: value is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__gt__(_W())  # value: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____iadd____value_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____iadd____value_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____iadd____value_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.__iadd__(value: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__iadd__(value: ReadableBuffer); call it with the wrong type.

typeshed contract: value is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__iadd__(_W())  # value: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____imul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____imul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____imul____value_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.__imul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__imul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__imul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____le____value_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____le____value_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____le____value_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.__le__(value: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__le__(value: ReadableBuffer); call it with the wrong type.

typeshed contract: value is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__le__(_W())  # value: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____lt____value_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____lt____value_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____lt____value_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.__lt__(value: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__lt__(value: ReadableBuffer); call it with the wrong type.

typeshed contract: value is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__lt__(_W())  # value: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____mul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____mul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____mul____value_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.__mul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__mul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__mul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____release_buffer____buffer_as_memoryview_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____release_buffer____buffer_as_memoryview_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____release_buffer____buffer_as_memoryview_wrong"
# subject = "builtins.bytearray.__release_buffer__(buffer: memoryview)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__release_buffer__(buffer: memoryview); call it with the wrong type.

typeshed contract: buffer is memoryview. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytearray
obj = bytearray()
try:
    obj.__release_buffer__(12345)  # buffer: memoryview <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____rmul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____rmul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____rmul____value_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.__rmul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__rmul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.__rmul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____setitem____key_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____setitem____key_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____setitem____key_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.__setitem__(key: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__setitem__(key: SupportsIndex); call it with the wrong type.

typeshed contract: key is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray()
try:
    obj.__setitem__(_W(), None)  # key: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray____setitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray____setitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray____setitem____key_as_slice_wrong"
# subject = "builtins.bytearray.__setitem__(key: slice)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__setitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray()
try:
    obj.__setitem__(_W(), None)  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__append__item_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__append__item_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__append__item_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.append(item: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.append(item: SupportsIndex); call it with the wrong type.

typeshed contract: item is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.append(_W())  # item: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__center__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__center__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__center__width_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.center(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.center(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.center(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__count__sub_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__count__sub_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__count__sub_as_typed_wrong"
# subject = "builtins.bytearray.count(sub: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.count(sub: typed); call it with the wrong type.

typeshed contract: sub is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.count(_W())  # sub: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__decode__encoding_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__decode__encoding_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__decode__encoding_as_str_wrong"
# subject = "builtins.bytearray.decode(encoding: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.decode(encoding: str); call it with the wrong type.

typeshed contract: encoding is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.decode(12345)  # encoding: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__endswith__suffix_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__endswith__suffix_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__endswith__suffix_as_typed_wrong"
# subject = "builtins.bytearray.endswith(suffix: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.endswith(suffix: typed); call it with the wrong type.

typeshed contract: suffix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray()
try:
    obj.endswith(_W())  # suffix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__expandtabs__tabsize_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__expandtabs__tabsize_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__expandtabs__tabsize_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.expandtabs(tabsize: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.expandtabs(tabsize: SupportsIndex); call it with the wrong type.

typeshed contract: tabsize is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.expandtabs(_W())  # tabsize: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__extend__iterable_of_ints_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__extend__iterable_of_ints_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__extend__iterable_of_ints_as_Iterable_wrong"
# subject = "builtins.bytearray.extend(iterable_of_ints: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.extend(iterable_of_ints: Iterable); call it with the wrong type.

typeshed contract: iterable_of_ints is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.extend(_W())  # iterable_of_ints: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__find__sub_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__find__sub_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__find__sub_as_typed_wrong"
# subject = "builtins.bytearray.find(sub: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.find(sub: typed); call it with the wrong type.

typeshed contract: sub is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.find(_W())  # sub: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__fromhex__string_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__fromhex__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__fromhex__string_as_str_wrong"
# subject = "builtins.bytearray.fromhex(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.fromhex(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytearray
try:
    bytearray.fromhex(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__fromhex__string_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__fromhex__string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__fromhex__string_as_typed_wrong"
# subject = "builtins.bytearray.fromhex(string: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.fromhex(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
try:
    bytearray.fromhex(_W())  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__hex__sep_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__hex__sep_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__hex__sep_as_typed_wrong"
# subject = "builtins.bytearray.hex(sep: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.hex(sep: typed); call it with the wrong type.

typeshed contract: sep is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.hex(_W())  # sep: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__index__sub_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__index__sub_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__index__sub_as_typed_wrong"
# subject = "builtins.bytearray.index(sub: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.index(sub: typed); call it with the wrong type.

typeshed contract: sub is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.index(_W())  # sub: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__init__ints_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__init__ints_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__init__ints_as_typed_wrong"
# subject = "builtins.bytearray.__init__(ints: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__init__(ints: typed); call it with the wrong type.

typeshed contract: ints is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
try:
    bytearray(_W())  # ints: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__init__string_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__init__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__init__string_as_str_wrong"
# subject = "builtins.bytearray.__init__(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.__init__(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytearray
try:
    bytearray(12345, "")  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__insert__index_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__insert__index_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__insert__index_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.insert(index: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.insert(index: SupportsIndex); call it with the wrong type.

typeshed contract: index is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.insert(_W(), None)  # index: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__join__iterable_of_bytes_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__join__iterable_of_bytes_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__join__iterable_of_bytes_as_Iterable_wrong"
# subject = "builtins.bytearray.join(iterable_of_bytes: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.join(iterable_of_bytes: Iterable); call it with the wrong type.

typeshed contract: iterable_of_bytes is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.join(_W())  # iterable_of_bytes: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__ljust__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__ljust__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__ljust__width_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.ljust(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.ljust(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.ljust(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__lstrip__bytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__lstrip__bytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__lstrip__bytes_as_typed_wrong"
# subject = "builtins.bytearray.lstrip(bytes: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.lstrip(bytes: typed); call it with the wrong type.

typeshed contract: bytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.lstrip(_W())  # bytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__maketrans__frm_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__maketrans__frm_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__maketrans__frm_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.maketrans(frm: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.maketrans(frm: ReadableBuffer); call it with the wrong type.

typeshed contract: frm is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
try:
    bytearray.maketrans(_W(), None)  # frm: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__partition__sep_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__partition__sep_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__partition__sep_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.partition(sep: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.partition(sep: ReadableBuffer); call it with the wrong type.

typeshed contract: sep is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.partition(_W())  # sep: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__pop__index_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__pop__index_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__pop__index_as_int_wrong"
# subject = "builtins.bytearray.pop(index: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.pop(index: int); call it with the wrong type.

typeshed contract: index is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.pop("not_an_int")  # index: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__remove__value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__remove__value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__remove__value_as_int_wrong"
# subject = "builtins.bytearray.remove(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.remove(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.remove("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__removeprefix__prefix_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__removeprefix__prefix_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__removeprefix__prefix_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.removeprefix(prefix: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.removeprefix(prefix: ReadableBuffer); call it with the wrong type.

typeshed contract: prefix is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.removeprefix(_W())  # prefix: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__removesuffix__suffix_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__removesuffix__suffix_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__removesuffix__suffix_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.removesuffix(suffix: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.removesuffix(suffix: ReadableBuffer); call it with the wrong type.

typeshed contract: suffix is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.removesuffix(_W())  # suffix: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__replace__old_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__replace__old_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__replace__old_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.replace(old: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.replace(old: ReadableBuffer); call it with the wrong type.

typeshed contract: old is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.replace(_W(), None)  # old: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__resize__size_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__resize__size_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__resize__size_as_int_wrong"
# subject = "builtins.bytearray.resize(size: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.resize(size: int); call it with the wrong type.

typeshed contract: size is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytearray
obj = object.__new__(bytearray)
try:
    obj.resize("not_an_int")  # size: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__rfind__sub_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__rfind__sub_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__rfind__sub_as_typed_wrong"
# subject = "builtins.bytearray.rfind(sub: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.rfind(sub: typed); call it with the wrong type.

typeshed contract: sub is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.rfind(_W())  # sub: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__rindex__sub_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__rindex__sub_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__rindex__sub_as_typed_wrong"
# subject = "builtins.bytearray.rindex(sub: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.rindex(sub: typed); call it with the wrong type.

typeshed contract: sub is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.rindex(_W())  # sub: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__rjust__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__rjust__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__rjust__width_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.rjust(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.rjust(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.rjust(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__rpartition__sep_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__rpartition__sep_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__rpartition__sep_as_ReadableBuffer_wrong"
# subject = "builtins.bytearray.rpartition(sep: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.rpartition(sep: ReadableBuffer); call it with the wrong type.

typeshed contract: sep is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.rpartition(_W())  # sep: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__rsplit__sep_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__rsplit__sep_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__rsplit__sep_as_typed_wrong"
# subject = "builtins.bytearray.rsplit(sep: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.rsplit(sep: typed); call it with the wrong type.

typeshed contract: sep is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.rsplit(_W())  # sep: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__rstrip__bytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__rstrip__bytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__rstrip__bytes_as_typed_wrong"
# subject = "builtins.bytearray.rstrip(bytes: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.rstrip(bytes: typed); call it with the wrong type.

typeshed contract: bytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.rstrip(_W())  # bytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__split__sep_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__split__sep_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__split__sep_as_typed_wrong"
# subject = "builtins.bytearray.split(sep: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.split(sep: typed); call it with the wrong type.

typeshed contract: sep is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.split(_W())  # sep: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__splitlines__keepends_as_bool_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__splitlines__keepends_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__splitlines__keepends_as_bool_wrong"
# subject = "builtins.bytearray.splitlines(keepends: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.splitlines(keepends: bool); call it with the wrong type.

typeshed contract: keepends is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytearray
obj = bytearray()
try:
    obj.splitlines("not_a_bool")  # keepends: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__startswith__prefix_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__startswith__prefix_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__startswith__prefix_as_typed_wrong"
# subject = "builtins.bytearray.startswith(prefix: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.startswith(prefix: typed); call it with the wrong type.

typeshed contract: prefix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray()
try:
    obj.startswith(_W())  # prefix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__strip__bytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__strip__bytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__strip__bytes_as_typed_wrong"
# subject = "builtins.bytearray.strip(bytes: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.strip(bytes: typed); call it with the wrong type.

typeshed contract: bytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.strip(_W())  # bytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__take_bytes__n_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__take_bytes__n_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__take_bytes__n_as_typed_wrong"
# subject = "builtins.bytearray.take_bytes(n: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.take_bytes(n: typed); call it with the wrong type.

typeshed contract: n is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = object.__new__(bytearray)
try:
    obj.take_bytes(_W())  # n: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__translate__table_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__translate__table_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__translate__table_as_typed_wrong"
# subject = "builtins.bytearray.translate(table: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.translate(table: typed); call it with the wrong type.

typeshed contract: table is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.translate(_W())  # table: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytearray__zfill__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytearray__zfill__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytearray__zfill__width_as_SupportsIndex_wrong"
# subject = "builtins.bytearray.zfill(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytearray.zfill(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytearray
obj = bytearray(b"abc")
try:
    obj.zfill(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____add____value_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____add____value_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____add____value_as_ReadableBuffer_wrong"
# subject = "builtins.bytes.__add__(value: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__add__(value: ReadableBuffer); call it with the wrong type.

typeshed contract: value is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.__add__(_W())  # value: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____buffer____flags_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____buffer____flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____buffer____flags_as_int_wrong"
# subject = "builtins.bytes.__buffer__(flags: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__buffer__(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytes
obj = b"abc"
try:
    obj.__buffer__("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____contains____key_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____contains____key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____contains____key_as_typed_wrong"
# subject = "builtins.bytes.__contains__(key: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__contains__(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.__contains__(_W())  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____ge____value_as_bytes_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____ge____value_as_bytes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____ge____value_as_bytes_wrong"
# subject = "builtins.bytes.__ge__(value: bytes)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__ge__(value: bytes); call it with the wrong type.

typeshed contract: value is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytes
obj = bytes()
try:
    obj.__ge__(12345)  # value: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____getitem____key_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____getitem____key_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____getitem____key_as_SupportsIndex_wrong"
# subject = "builtins.bytes.__getitem__(key: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__getitem__(key: SupportsIndex); call it with the wrong type.

typeshed contract: key is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = bytes()
try:
    obj.__getitem__(_W())  # key: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____getitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____getitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____getitem____key_as_slice_wrong"
# subject = "builtins.bytes.__getitem__(key: slice)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__getitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = bytes()
try:
    obj.__getitem__(_W())  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____gt____value_as_bytes_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____gt____value_as_bytes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____gt____value_as_bytes_wrong"
# subject = "builtins.bytes.__gt__(value: bytes)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__gt__(value: bytes); call it with the wrong type.

typeshed contract: value is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytes
obj = bytes()
try:
    obj.__gt__(12345)  # value: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____le____value_as_bytes_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____le____value_as_bytes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____le____value_as_bytes_wrong"
# subject = "builtins.bytes.__le__(value: bytes)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__le__(value: bytes); call it with the wrong type.

typeshed contract: value is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytes
obj = bytes()
try:
    obj.__le__(12345)  # value: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____lt____value_as_bytes_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____lt____value_as_bytes_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____lt____value_as_bytes_wrong"
# subject = "builtins.bytes.__lt__(value: bytes)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__lt__(value: bytes); call it with the wrong type.

typeshed contract: value is bytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytes
obj = bytes()
try:
    obj.__lt__(12345)  # value: bytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____mul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____mul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____mul____value_as_SupportsIndex_wrong"
# subject = "builtins.bytes.__mul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__mul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.__mul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____new____o_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____new____o_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____new____o_as_typed_wrong"
# subject = "builtins.bytes.__new__(o: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__new__(o: typed); call it with the wrong type.

typeshed contract: o is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
try:
    bytes(_W())  # bytes.__new__ o: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____new____string_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____new____string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____new____string_as_str_wrong"
# subject = "builtins.bytes.__new__(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__new__(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytes
try:
    bytes(12345, "")  # bytes.__new__ string: str <- wrong-typed when encoding is present
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes____rmul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes____rmul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes____rmul____value_as_SupportsIndex_wrong"
# subject = "builtins.bytes.__rmul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.__rmul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.__rmul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__center__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__center__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__center__width_as_SupportsIndex_wrong"
# subject = "builtins.bytes.center(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.center(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.center(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__count__sub_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__count__sub_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__count__sub_as_typed_wrong"
# subject = "builtins.bytes.count(sub: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.count(sub: typed); call it with the wrong type.

typeshed contract: sub is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.count(_W())  # sub: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__decode__encoding_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__decode__encoding_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__decode__encoding_as_str_wrong"
# subject = "builtins.bytes.decode(encoding: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.decode(encoding: str); call it with the wrong type.

typeshed contract: encoding is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytes
obj = b"abc"
try:
    obj.decode(12345)  # encoding: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__endswith__suffix_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__endswith__suffix_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__endswith__suffix_as_typed_wrong"
# subject = "builtins.bytes.endswith(suffix: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.endswith(suffix: typed); call it with the wrong type.

typeshed contract: suffix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = bytes()
try:
    obj.endswith(_W())  # suffix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__expandtabs__tabsize_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__expandtabs__tabsize_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__expandtabs__tabsize_as_SupportsIndex_wrong"
# subject = "builtins.bytes.expandtabs(tabsize: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.expandtabs(tabsize: SupportsIndex); call it with the wrong type.

typeshed contract: tabsize is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.expandtabs(_W())  # tabsize: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__find__sub_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__find__sub_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__find__sub_as_typed_wrong"
# subject = "builtins.bytes.find(sub: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.find(sub: typed); call it with the wrong type.

typeshed contract: sub is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.find(_W())  # sub: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__fromhex__string_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__fromhex__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__fromhex__string_as_str_wrong"
# subject = "builtins.bytes.fromhex(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.fromhex(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytes
try:
    bytes.fromhex(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__fromhex__string_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__fromhex__string_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__fromhex__string_as_typed_wrong"
# subject = "builtins.bytes.fromhex(string: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.fromhex(string: typed); call it with the wrong type.

typeshed contract: string is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
try:
    bytes.fromhex(_W())  # string: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__hex__sep_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__hex__sep_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__hex__sep_as_typed_wrong"
# subject = "builtins.bytes.hex(sep: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.hex(sep: typed); call it with the wrong type.

typeshed contract: sep is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.hex(_W())  # sep: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__index__sub_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__index__sub_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__index__sub_as_typed_wrong"
# subject = "builtins.bytes.index(sub: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.index(sub: typed); call it with the wrong type.

typeshed contract: sub is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.index(_W())  # sub: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__join__iterable_of_bytes_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__join__iterable_of_bytes_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__join__iterable_of_bytes_as_Iterable_wrong"
# subject = "builtins.bytes.join(iterable_of_bytes: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.join(iterable_of_bytes: Iterable); call it with the wrong type.

typeshed contract: iterable_of_bytes is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.join(_W())  # iterable_of_bytes: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__ljust__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__ljust__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__ljust__width_as_SupportsIndex_wrong"
# subject = "builtins.bytes.ljust(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.ljust(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.ljust(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__lstrip__bytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__lstrip__bytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__lstrip__bytes_as_typed_wrong"
# subject = "builtins.bytes.lstrip(bytes: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.lstrip(bytes: typed); call it with the wrong type.

typeshed contract: bytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.lstrip(_W())  # bytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__maketrans__frm_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__maketrans__frm_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__maketrans__frm_as_ReadableBuffer_wrong"
# subject = "builtins.bytes.maketrans(frm: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.maketrans(frm: ReadableBuffer); call it with the wrong type.

typeshed contract: frm is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
try:
    bytes.maketrans(_W(), None)  # frm: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__partition__sep_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__partition__sep_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__partition__sep_as_ReadableBuffer_wrong"
# subject = "builtins.bytes.partition(sep: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.partition(sep: ReadableBuffer); call it with the wrong type.

typeshed contract: sep is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.partition(_W())  # sep: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__removeprefix__prefix_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__removeprefix__prefix_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__removeprefix__prefix_as_ReadableBuffer_wrong"
# subject = "builtins.bytes.removeprefix(prefix: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.removeprefix(prefix: ReadableBuffer); call it with the wrong type.

typeshed contract: prefix is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.removeprefix(_W())  # prefix: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__removesuffix__suffix_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__removesuffix__suffix_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__removesuffix__suffix_as_ReadableBuffer_wrong"
# subject = "builtins.bytes.removesuffix(suffix: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.removesuffix(suffix: ReadableBuffer); call it with the wrong type.

typeshed contract: suffix is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.removesuffix(_W())  # suffix: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__replace__old_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__replace__old_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__replace__old_as_ReadableBuffer_wrong"
# subject = "builtins.bytes.replace(old: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.replace(old: ReadableBuffer); call it with the wrong type.

typeshed contract: old is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.replace(_W(), None)  # old: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__rfind__sub_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__rfind__sub_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__rfind__sub_as_typed_wrong"
# subject = "builtins.bytes.rfind(sub: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.rfind(sub: typed); call it with the wrong type.

typeshed contract: sub is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.rfind(_W())  # sub: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__rindex__sub_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__rindex__sub_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__rindex__sub_as_typed_wrong"
# subject = "builtins.bytes.rindex(sub: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.rindex(sub: typed); call it with the wrong type.

typeshed contract: sub is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.rindex(_W())  # sub: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__rjust__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__rjust__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__rjust__width_as_SupportsIndex_wrong"
# subject = "builtins.bytes.rjust(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.rjust(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.rjust(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__rpartition__sep_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__rpartition__sep_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__rpartition__sep_as_ReadableBuffer_wrong"
# subject = "builtins.bytes.rpartition(sep: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.rpartition(sep: ReadableBuffer); call it with the wrong type.

typeshed contract: sep is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.rpartition(_W())  # sep: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__rsplit__sep_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__rsplit__sep_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__rsplit__sep_as_typed_wrong"
# subject = "builtins.bytes.rsplit(sep: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.rsplit(sep: typed); call it with the wrong type.

typeshed contract: sep is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.rsplit(_W())  # sep: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__rstrip__bytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__rstrip__bytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__rstrip__bytes_as_typed_wrong"
# subject = "builtins.bytes.rstrip(bytes: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.rstrip(bytes: typed); call it with the wrong type.

typeshed contract: bytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.rstrip(_W())  # bytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__split__sep_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__split__sep_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__split__sep_as_typed_wrong"
# subject = "builtins.bytes.split(sep: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.split(sep: typed); call it with the wrong type.

typeshed contract: sep is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.split(_W())  # sep: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__splitlines__keepends_as_bool_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__splitlines__keepends_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__splitlines__keepends_as_bool_wrong"
# subject = "builtins.bytes.splitlines(keepends: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.splitlines(keepends: bool); call it with the wrong type.

typeshed contract: keepends is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import bytes
obj = bytes()
try:
    obj.splitlines("not_a_bool")  # keepends: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__startswith__prefix_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__startswith__prefix_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__startswith__prefix_as_typed_wrong"
# subject = "builtins.bytes.startswith(prefix: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.startswith(prefix: typed); call it with the wrong type.

typeshed contract: prefix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = bytes()
try:
    obj.startswith(_W())  # prefix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__strip__bytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__strip__bytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__strip__bytes_as_typed_wrong"
# subject = "builtins.bytes.strip(bytes: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.strip(bytes: typed); call it with the wrong type.

typeshed contract: bytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.strip(_W())  # bytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__translate__table_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__translate__table_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__translate__table_as_typed_wrong"
# subject = "builtins.bytes.translate(table: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.translate(table: typed); call it with the wrong type.

typeshed contract: table is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.translate(_W())  # table: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/bytes__zfill__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_bytes__zfill__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "bytes__zfill__width_as_SupportsIndex_wrong"
# subject = "builtins.bytes.zfill(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.bytes.zfill(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import bytes
obj = b"abc"
try:
    obj.zfill(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/chr__i_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_chr__i_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "chr__i_as_SupportsIndex_wrong"
# subject = "builtins.chr(i: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.chr(i: SupportsIndex); call it with the wrong type.

typeshed contract: i is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    chr(_W())  # i: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/chr__i_rejects_keyword_argument.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_chr__i_rejects_keyword_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "chr__i_rejects_keyword_argument"
# subject = "builtins.chr(i: SupportsIndex, /)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.chr(i, /) is positional-only in CPython — calling it
with a keyword, `chr(i=65)`, raises `TypeError: chr() takes no keyword
arguments` even though `65` is a perfectly well-typed `SupportsIndex` value.

The generated TypeSpec contract retains `/` as `ParamSpecKind::PosOnly`, and
its structured binder is authoritative for call shape even though runtime
builtin registration also exposes a `Ty::Fn`. The companion case
`chr__i_as_SupportsIndex_wrong.py` covers the argument type contract; this
fixture independently locks positional-only binding."""

try:
    chr(i=65)  # chr is positional-only; ANY keyword arg is rejected
    print("no_typeerror:")  # CPython accepted the keyword arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/classmethod____get____instance_as__T_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_classmethod____get____instance_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "classmethod____get____instance_as__T_wrong"
# subject = "builtins.classmethod.__get__(instance: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.classmethod.__get__(instance: _T); call it with the wrong type.

typeshed contract: instance is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import classmethod
obj = classmethod(lambda cls: None)
try:
    obj.__get__(_W())  # instance: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/classmethod____get____instance_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_classmethod____get____instance_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "classmethod____get____instance_as_typed_wrong"
# subject = "builtins.classmethod.__get__(instance: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.classmethod.__get__(instance: typed); call it with the wrong type.

typeshed contract: instance is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import classmethod
obj = classmethod(lambda cls: None)
try:
    obj.__get__(_W(), None)  # instance: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/classmethod__init__f_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_classmethod__init__f_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "classmethod__init__f_as_Callable_wrong"
# subject = "builtins.classmethod.__init__(f: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.classmethod.__init__(f: Callable); call it with the wrong type.

typeshed contract: f is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import classmethod
try:
    classmethod(_W())  # f: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/compile__source_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_compile__source_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "compile__source_as_typed_wrong"
# subject = "builtins.compile(source: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.compile(source: typed); call it with the wrong type.

typeshed contract: source is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    compile(_W(), None, "", None)  # source: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____add____value_as_complex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____add____value_as_complex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____add____value_as_complex_wrong"
# subject = "builtins.complex.__add__(value: complex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__add__(value: complex); call it with the wrong type.

typeshed contract: value is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import complex
obj = complex()
try:
    obj.__add__("not_a_complex")  # value: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____format____format_spec_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____format____format_spec_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____format____format_spec_as_str_wrong"
# subject = "builtins.complex.__format__(format_spec: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__format__(format_spec: str); call it with the wrong type.

typeshed contract: format_spec is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import complex
obj = complex()
try:
    obj.__format__(12345)  # format_spec: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____mul____value_as_complex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____mul____value_as_complex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____mul____value_as_complex_wrong"
# subject = "builtins.complex.__mul__(value: complex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__mul__(value: complex); call it with the wrong type.

typeshed contract: value is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import complex
obj = complex()
try:
    obj.__mul__("not_a_complex")  # value: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____new____real_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____new____real_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____new____real_as_typed_wrong"
# subject = "builtins.complex.__new__(real: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__new__(real: typed); call it with the wrong type.

typeshed contract: real is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import complex
try:
    complex(_W())  # complex.__new__ real: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____pow____value_as_complex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____pow____value_as_complex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____pow____value_as_complex_wrong"
# subject = "builtins.complex.__pow__(value: complex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__pow__(value: complex); call it with the wrong type.

typeshed contract: value is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import complex
obj = complex()
try:
    obj.__pow__("not_a_complex")  # value: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____radd____value_as_complex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____radd____value_as_complex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____radd____value_as_complex_wrong"
# subject = "builtins.complex.__radd__(value: complex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__radd__(value: complex); call it with the wrong type.

typeshed contract: value is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import complex
obj = complex()
try:
    obj.__radd__("not_a_complex")  # value: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____rmul____value_as_complex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____rmul____value_as_complex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____rmul____value_as_complex_wrong"
# subject = "builtins.complex.__rmul__(value: complex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__rmul__(value: complex); call it with the wrong type.

typeshed contract: value is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import complex
obj = complex()
try:
    obj.__rmul__("not_a_complex")  # value: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____rpow____value_as_complex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____rpow____value_as_complex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____rpow____value_as_complex_wrong"
# subject = "builtins.complex.__rpow__(value: complex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__rpow__(value: complex); call it with the wrong type.

typeshed contract: value is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import complex
obj = complex()
try:
    obj.__rpow__("not_a_complex")  # value: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____rsub____value_as_complex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____rsub____value_as_complex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____rsub____value_as_complex_wrong"
# subject = "builtins.complex.__rsub__(value: complex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__rsub__(value: complex); call it with the wrong type.

typeshed contract: value is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import complex
obj = complex()
try:
    obj.__rsub__("not_a_complex")  # value: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____rtruediv____value_as_complex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____rtruediv____value_as_complex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____rtruediv____value_as_complex_wrong"
# subject = "builtins.complex.__rtruediv__(value: complex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__rtruediv__(value: complex); call it with the wrong type.

typeshed contract: value is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import complex
obj = complex()
try:
    obj.__rtruediv__("not_a_complex")  # value: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____sub____value_as_complex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____sub____value_as_complex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____sub____value_as_complex_wrong"
# subject = "builtins.complex.__sub__(value: complex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__sub__(value: complex); call it with the wrong type.

typeshed contract: value is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import complex
obj = complex()
try:
    obj.__sub__("not_a_complex")  # value: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex____truediv____value_as_complex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex____truediv____value_as_complex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex____truediv____value_as_complex_wrong"
# subject = "builtins.complex.__truediv__(value: complex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.__truediv__(value: complex); call it with the wrong type.

typeshed contract: value is complex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import complex
obj = complex()
try:
    obj.__truediv__("not_a_complex")  # value: complex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/complex__from_number__number_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_complex__from_number__number_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "complex__from_number__number_as_typed_wrong"
# subject = "builtins.complex.from_number(number: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.complex.from_number(number: typed); call it with the wrong type.

typeshed contract: number is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import complex
try:
    complex.from_number(_W())  # number: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/delattr__name_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_delattr__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "delattr__name_as_str_wrong"
# subject = "builtins.delattr(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.delattr(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

try:
    delattr(None, 12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict____delitem____key_as__KT_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict____delitem____key_as__KT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict____delitem____key_as__KT_wrong"
# subject = "builtins.dict.__delitem__(key: _KT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__delitem__(key: _KT); call it with the wrong type.

typeshed contract: key is _KT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
obj: dict[str, int] = {}
try:
    obj.__delitem__(_W())  # key: _KT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except KeyError:
    print("no_typeerror:")  # CPython accepted the key type and performed lookup/delete
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict____getitem____key_as__KT_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict____getitem____key_as__KT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict____getitem____key_as__KT_wrong"
# subject = "builtins.dict.__getitem__(key: _KT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__getitem__(key: _KT); call it with the wrong type.

typeshed contract: key is _KT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
obj: dict[str, int] = {}
try:
    obj.__getitem__(_W())  # key: _KT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except KeyError:
    print("no_typeerror:")  # CPython accepted the key type and performed lookup
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict____ior____value_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict____ior____value_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict____ior____value_as_Iterable_wrong"
# subject = "builtins.dict.__ior__(value: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__ior__(value: Iterable); call it with the wrong type.

typeshed contract: value is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
obj: dict[str, int] = {}
try:
    obj.__ior__(_W())  # value: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict____ior____value_as_SupportsKeysAndGetItem_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict____ior____value_as_SupportsKeysAndGetItem_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict____ior____value_as_SupportsKeysAndGetItem_wrong"
# subject = "builtins.dict.__ior__(value: SupportsKeysAndGetItem)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__ior__(value: SupportsKeysAndGetItem); call it with the wrong type.

typeshed contract: value is SupportsKeysAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
obj: dict[str, int] = {}
try:
    obj.__ior__(_W())  # value: SupportsKeysAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict____or____value_as_dict_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict____or____value_as_dict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict____or____value_as_dict_wrong"
# subject = "builtins.dict.__or__(value: dict)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__or__(value: dict); call it with the wrong type.

typeshed contract: value is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import dict
obj: dict[str, int] = {}
try:
    obj.__or__(12345)  # value: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict____or____value_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict____or____value_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict____or____value_as_typed_wrong"
# subject = "builtins.dict.__or__(value: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__or__(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
obj: dict[str, int] = {}
try:
    obj.__or__(_W())  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict____ror____value_as_dict_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict____ror____value_as_dict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict____ror____value_as_dict_wrong"
# subject = "builtins.dict.__ror__(value: dict)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__ror__(value: dict); call it with the wrong type.

typeshed contract: value is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import dict
obj: dict[str, int] = {}
try:
    obj.__ror__(12345)  # value: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict____ror____value_as_frozendict_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict____ror____value_as_frozendict_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict____ror____value_as_frozendict_wrong"
# subject = "builtins.dict.__ror__(value: frozendict)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__ror__(value: frozendict); call it with the wrong type.

typeshed contract: value is frozendict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
obj: dict[str, int] = {}
try:
    obj.__ror__(_W())  # value: frozendict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict____setitem____key_as__KT_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict____setitem____key_as__KT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict____setitem____key_as__KT_wrong"
# subject = "builtins.dict.__setitem__(key: _KT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__setitem__(key: _KT); call it with the wrong type.

typeshed contract: key is _KT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
obj: dict[str, int] = {}
try:
    obj.__setitem__(_W(), 1)  # key: _KT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict__fromkeys__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict__fromkeys__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict__fromkeys__iterable_as_Iterable_wrong"
# subject = "builtins.dict.fromkeys(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.fromkeys(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
try:
    dict.fromkeys(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict__get__key_as__KT_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict__get__key_as__KT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict__get__key_as__KT_wrong"
# subject = "builtins.dict.get(key: _KT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.get(key: _KT); call it with the wrong type.

typeshed contract: key is _KT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
obj: dict[str, int] = {}
try:
    obj.get(_W())  # key: _KT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict__init__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict__init__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict__init__iterable_as_Iterable_wrong"
# subject = "builtins.dict.__init__(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__init__(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
try:
    dict(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict__init__map_as_SupportsKeysAndGetItem_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict__init__map_as_SupportsKeysAndGetItem_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict__init__map_as_SupportsKeysAndGetItem_wrong"
# subject = "builtins.dict.__init__(map: SupportsKeysAndGetItem)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.__init__(map: SupportsKeysAndGetItem); call it with the wrong type.

typeshed contract: map is SupportsKeysAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
try:
    dict(_W())  # map: SupportsKeysAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/dict__pop__key_as__KT_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_dict__pop__key_as__KT_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "dict__pop__key_as__KT_wrong"
# subject = "builtins.dict.pop(key: _KT)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.dict.pop(key: _KT); call it with the wrong type.

typeshed contract: key is _KT. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import dict
obj: dict[str, int] = {}
try:
    obj.pop(_W(), None)  # key: _KT <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/enumerate____new____iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_enumerate____new____iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "enumerate____new____iterable_as_Iterable_wrong"
# subject = "builtins.enumerate.__new__(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.enumerate.__new__(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import enumerate
obj = object.__new__(enumerate)
try:
    obj.__new__(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/eval__source_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_eval__source_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "eval__source_as_typed_wrong"
# subject = "builtins.eval(source: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.eval(source: typed); call it with the wrong type.

typeshed contract: source is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    eval(_W())  # source: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/exec__source_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_exec__source_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "exec__source_as_typed_wrong"
# subject = "builtins.exec(source: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.exec(source: typed); call it with the wrong type.

typeshed contract: source is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    exec(_W())  # source: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/filter____new____function_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_filter____new____function_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "filter____new____function_as_Callable_wrong"
# subject = "builtins.filter.__new__(function: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.filter.__new__(function: Callable); call it with the wrong type.

typeshed contract: function is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import filter
try:
    filter(_W(), [])  # filter.__new__ function: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/filter____new____function_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_filter____new____function_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "filter____new____function_as_typed_wrong"
# subject = "builtins.filter.__new__(function: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.filter.__new__(function: typed); call it with the wrong type.

typeshed contract: function is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import filter
try:
    filter(_W(), [])  # filter.__new__ function: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____add____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____add____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____add____value_as_float_wrong"
# subject = "builtins.float.__add__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__add__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__add__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____divmod____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____divmod____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____divmod____value_as_float_wrong"
# subject = "builtins.float.__divmod__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__divmod__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__divmod__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____floordiv____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____floordiv____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____floordiv____value_as_float_wrong"
# subject = "builtins.float.__floordiv__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__floordiv__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__floordiv__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____format____format_spec_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____format____format_spec_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____format____format_spec_as_str_wrong"
# subject = "builtins.float.__format__(format_spec: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__format__(format_spec: str); call it with the wrong type.

typeshed contract: format_spec is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__format__(12345)  # format_spec: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____ge____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____ge____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____ge____value_as_float_wrong"
# subject = "builtins.float.__ge__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__ge__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__ge__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____gt____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____gt____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____gt____value_as_float_wrong"
# subject = "builtins.float.__gt__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__gt__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__gt__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____le____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____le____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____le____value_as_float_wrong"
# subject = "builtins.float.__le__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__le__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__le__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____lt____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____lt____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____lt____value_as_float_wrong"
# subject = "builtins.float.__lt__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__lt__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__lt__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____mod____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____mod____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____mod____value_as_float_wrong"
# subject = "builtins.float.__mod__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__mod__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__mod__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____mul____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____mul____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____mul____value_as_float_wrong"
# subject = "builtins.float.__mul__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__mul__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__mul__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____new____x_as_ConvertibleToFloat_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____new____x_as_ConvertibleToFloat_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____new____x_as_ConvertibleToFloat_wrong"
# subject = "builtins.float.__new__(x: ConvertibleToFloat)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__new__(x: ConvertibleToFloat); call it with the wrong type.

typeshed contract: x is ConvertibleToFloat. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import float
obj = object.__new__(float)
try:
    obj.__new__(_W())  # x: ConvertibleToFloat <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____pow____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____pow____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____pow____value_as_float_wrong"
# subject = "builtins.float.__pow__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__pow__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = float()
try:
    obj.__pow__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____pow____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____pow____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____pow____value_as_int_wrong"
# subject = "builtins.float.__pow__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__pow__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = float()
try:
    obj.__pow__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____radd____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____radd____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____radd____value_as_float_wrong"
# subject = "builtins.float.__radd__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__radd__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__radd__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____rdivmod____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____rdivmod____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____rdivmod____value_as_float_wrong"
# subject = "builtins.float.__rdivmod__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__rdivmod__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__rdivmod__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____rfloordiv____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____rfloordiv____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____rfloordiv____value_as_float_wrong"
# subject = "builtins.float.__rfloordiv__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__rfloordiv__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__rfloordiv__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____rmod____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____rmod____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____rmod____value_as_float_wrong"
# subject = "builtins.float.__rmod__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__rmod__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__rmod__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____rmul____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____rmul____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____rmul____value_as_float_wrong"
# subject = "builtins.float.__rmul__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__rmul__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__rmul__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____round____ndigits_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____round____ndigits_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____round____ndigits_as_SupportsIndex_wrong"
# subject = "builtins.float.__round__(ndigits: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__round__(ndigits: SupportsIndex); call it with the wrong type.

typeshed contract: ndigits is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import float
obj = float()
try:
    obj.__round__(_W())  # ndigits: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____round____ndigits_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____round____ndigits_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____round____ndigits_as_typed_wrong"
# subject = "builtins.float.__round__(ndigits: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__round__(ndigits: typed); call it with the wrong type.

typeshed contract: ndigits is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import float
obj = float()
try:
    obj.__round__(_W())  # ndigits: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____rpow____value_as__NegativeInteger_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____rpow____value_as__NegativeInteger_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____rpow____value_as__NegativeInteger_wrong"
# subject = "builtins.float.__rpow__(value: _NegativeInteger)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__rpow__(value: _NegativeInteger); call it with the wrong type.

typeshed contract: value is _NegativeInteger. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import float
obj = float()
try:
    obj.__rpow__(_W())  # value: _NegativeInteger <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____rpow____value_as__PositiveInteger_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____rpow____value_as__PositiveInteger_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____rpow____value_as__PositiveInteger_wrong"
# subject = "builtins.float.__rpow__(value: _PositiveInteger)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__rpow__(value: _PositiveInteger); call it with the wrong type.

typeshed contract: value is _PositiveInteger. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import float
obj = float()
try:
    obj.__rpow__(_W())  # value: _PositiveInteger <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____rpow____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____rpow____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____rpow____value_as_float_wrong"
# subject = "builtins.float.__rpow__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__rpow__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = float()
try:
    obj.__rpow__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____rsub____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____rsub____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____rsub____value_as_float_wrong"
# subject = "builtins.float.__rsub__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__rsub__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__rsub__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____rtruediv____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____rtruediv____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____rtruediv____value_as_float_wrong"
# subject = "builtins.float.__rtruediv__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__rtruediv__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__rtruediv__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____sub____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____sub____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____sub____value_as_float_wrong"
# subject = "builtins.float.__sub__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__sub__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__sub__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float____truediv____value_as_float_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float____truediv____value_as_float_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float____truediv____value_as_float_wrong"
# subject = "builtins.float.__truediv__(value: float)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.__truediv__(value: float); call it with the wrong type.

typeshed contract: value is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
obj = object.__new__(float)
try:
    obj.__truediv__("not_a_float")  # value: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float__from_number__number_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float__from_number__number_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float__from_number__number_as_typed_wrong"
# subject = "builtins.float.from_number(number: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.from_number(number: typed); call it with the wrong type.

typeshed contract: number is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import float
try:
    float.from_number(_W())  # number: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/float__fromhex__string_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_float__fromhex__string_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "float__fromhex__string_as_str_wrong"
# subject = "builtins.float.fromhex(string: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.float.fromhex(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import float
try:
    float.fromhex(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/format__format_spec_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_format__format_spec_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "format__format_spec_as_str_wrong"
# subject = "builtins.format(format_spec: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.format(format_spec: str); call it with the wrong type.

typeshed contract: format_spec is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

try:
    format(None, 12345)  # format_spec: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset____and____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset____and____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset____and____value_as_AbstractSet_wrong"
# subject = "builtins.frozenset.__and__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.__and__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = frozenset()
try:
    obj.__and__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset____ge____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset____ge____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset____ge____value_as_AbstractSet_wrong"
# subject = "builtins.frozenset.__ge__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.__ge__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = frozenset()
try:
    obj.__ge__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset____gt____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset____gt____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset____gt____value_as_AbstractSet_wrong"
# subject = "builtins.frozenset.__gt__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.__gt__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = frozenset()
try:
    obj.__gt__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset____le____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset____le____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset____le____value_as_AbstractSet_wrong"
# subject = "builtins.frozenset.__le__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.__le__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = frozenset()
try:
    obj.__le__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset____lt____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset____lt____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset____lt____value_as_AbstractSet_wrong"
# subject = "builtins.frozenset.__lt__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.__lt__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = frozenset()
try:
    obj.__lt__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset____new____iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset____new____iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset____new____iterable_as_Iterable_wrong"
# subject = "builtins.frozenset.__new__(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.__new__(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
try:
    frozenset.__new__(frozenset, _W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset____or____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset____or____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset____or____value_as_AbstractSet_wrong"
# subject = "builtins.frozenset.__or__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.__or__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = frozenset()
try:
    obj.__or__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset____sub____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset____sub____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset____sub____value_as_AbstractSet_wrong"
# subject = "builtins.frozenset.__sub__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.__sub__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = frozenset()
try:
    obj.__sub__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset____xor____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset____xor____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset____xor____value_as_AbstractSet_wrong"
# subject = "builtins.frozenset.__xor__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.__xor__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = frozenset()
try:
    obj.__xor__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset__isdisjoint__s_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset__isdisjoint__s_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset__isdisjoint__s_as_Iterable_wrong"
# subject = "builtins.frozenset.isdisjoint(s: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.isdisjoint(s: Iterable); call it with the wrong type.

typeshed contract: s is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = object.__new__(frozenset)
try:
    obj.isdisjoint(_W())  # s: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset__issubset__s_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset__issubset__s_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset__issubset__s_as_Iterable_wrong"
# subject = "builtins.frozenset.issubset(s: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.issubset(s: Iterable); call it with the wrong type.

typeshed contract: s is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = object.__new__(frozenset)
try:
    obj.issubset(_W())  # s: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset__issuperset__s_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset__issuperset__s_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset__issuperset__s_as_Iterable_wrong"
# subject = "builtins.frozenset.issuperset(s: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.issuperset(s: Iterable); call it with the wrong type.

typeshed contract: s is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = object.__new__(frozenset)
try:
    obj.issuperset(_W())  # s: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/frozenset__symmetric_difference__s_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_frozenset__symmetric_difference__s_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "frozenset__symmetric_difference__s_as_Iterable_wrong"
# subject = "builtins.frozenset.symmetric_difference(s: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.frozenset.symmetric_difference(s: Iterable); call it with the wrong type.

typeshed contract: s is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import frozenset
obj = object.__new__(frozenset)
try:
    obj.symmetric_difference(_W())  # s: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/function____get____owner_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_function____get____owner_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "function____get____owner_as_typed_wrong"
# subject = "builtins.function.__get__(owner: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.function.__get__(owner: typed); call it with the wrong type.

typeshed contract: owner is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


def _function_probe():
    pass


try:
    _function_probe.__get__(None, _W())  # owner: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/function____new____code_as_CodeType_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_function____new____code_as_CodeType_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "function____new____code_as_CodeType_wrong"
# subject = "builtins.function.__new__(code: CodeType)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.function.__new__(code: CodeType); call it with the wrong type.

typeshed contract: code is CodeType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import function
obj = object.__new__(function)
try:
    obj.__new__(_W(), None)  # code: CodeType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/getattr__name_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_getattr__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "getattr__name_as_str_wrong"
# subject = "builtins.getattr(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.getattr(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

try:
    getattr(None, 12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/hasattr__name_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_hasattr__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "hasattr__name_as_str_wrong"
# subject = "builtins.hasattr(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.hasattr(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

try:
    hasattr(None, 12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/hex__number_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_hex__number_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "hex__number_as_SupportsIndex_wrong"
# subject = "builtins.hex(number: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.hex(number: SupportsIndex); call it with the wrong type.

typeshed contract: number is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    hex(_W())  # number: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____add____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____add____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____add____value_as_int_wrong"
# subject = "builtins.int.__add__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__add__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__add__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____and____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____and____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____and____value_as_int_wrong"
# subject = "builtins.int.__and__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__and__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__and__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____divmod____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____divmod____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____divmod____value_as_int_wrong"
# subject = "builtins.int.__divmod__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__divmod__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__divmod__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____floordiv____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____floordiv____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____floordiv____value_as_int_wrong"
# subject = "builtins.int.__floordiv__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__floordiv__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__floordiv__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____format____format_spec_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____format____format_spec_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____format____format_spec_as_str_wrong"
# subject = "builtins.int.__format__(format_spec: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__format__(format_spec: str); call it with the wrong type.

typeshed contract: format_spec is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__format__(12345)  # format_spec: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____ge____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____ge____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____ge____value_as_int_wrong"
# subject = "builtins.int.__ge__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__ge__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__ge__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____gt____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____gt____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____gt____value_as_int_wrong"
# subject = "builtins.int.__gt__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__gt__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__gt__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____le____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____le____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____le____value_as_int_wrong"
# subject = "builtins.int.__le__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__le__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__le__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____lshift____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____lshift____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____lshift____value_as_int_wrong"
# subject = "builtins.int.__lshift__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__lshift__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__lshift__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____lt____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____lt____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____lt____value_as_int_wrong"
# subject = "builtins.int.__lt__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__lt__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__lt__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____mod____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____mod____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____mod____value_as_int_wrong"
# subject = "builtins.int.__mod__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__mod__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__mod__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____mul____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____mul____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____mul____value_as_int_wrong"
# subject = "builtins.int.__mul__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__mul__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__mul__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____new____x_as_ConvertibleToInt_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____new____x_as_ConvertibleToInt_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____new____x_as_ConvertibleToInt_wrong"
# subject = "builtins.int.__new__(x: ConvertibleToInt)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__new__(x: ConvertibleToInt); call it with the wrong type.

typeshed contract: x is ConvertibleToInt. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import int
try:
    int.__new__(int, _W())  # x: ConvertibleToInt <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____new____x_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____new____x_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____new____x_as_typed_wrong"
# subject = "builtins.int.__new__(x: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__new__(x: typed); call it with the wrong type.

typeshed contract: x is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import int
try:
    int.__new__(int, _W(), None)  # x: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____or____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____or____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____or____value_as_int_wrong"
# subject = "builtins.int.__or__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__or__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__or__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____pow____value_as_Literal_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____pow____value_as_Literal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____pow____value_as_Literal_wrong"
# subject = "builtins.int.__pow__(value: Literal)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__pow__(value: Literal); call it with the wrong type.

typeshed contract: value is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import int
obj = int()
try:
    obj.__pow__(_W(), None)  # value: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____pow____value_as__NegativeInteger_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____pow____value_as__NegativeInteger_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____pow____value_as__NegativeInteger_wrong"
# subject = "builtins.int.__pow__(value: _NegativeInteger)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__pow__(value: _NegativeInteger); call it with the wrong type.

typeshed contract: value is _NegativeInteger. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import int
obj = int()
try:
    obj.__pow__(_W())  # value: _NegativeInteger <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____pow____value_as__PositiveInteger_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____pow____value_as__PositiveInteger_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____pow____value_as__PositiveInteger_wrong"
# subject = "builtins.int.__pow__(value: _PositiveInteger)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__pow__(value: _PositiveInteger); call it with the wrong type.

typeshed contract: value is _PositiveInteger. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import int
obj = int()
try:
    obj.__pow__(_W())  # value: _PositiveInteger <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____pow____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____pow____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____pow____value_as_int_wrong"
# subject = "builtins.int.__pow__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__pow__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = int()
try:
    obj.__pow__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____pow____x_as_Literal_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____pow____x_as_Literal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____pow____x_as_Literal_wrong"
# subject = "builtins.int.__pow__(x: Literal)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__pow__(x: Literal); call it with the wrong type.

typeshed contract: x is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import int
obj = int()
try:
    obj.__pow__(_W())  # x: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____radd____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____radd____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____radd____value_as_int_wrong"
# subject = "builtins.int.__radd__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__radd__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__radd__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rand____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rand____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rand____value_as_int_wrong"
# subject = "builtins.int.__rand__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rand__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rand__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rdivmod____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rdivmod____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rdivmod____value_as_int_wrong"
# subject = "builtins.int.__rdivmod__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rdivmod__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rdivmod__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rfloordiv____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rfloordiv____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rfloordiv____value_as_int_wrong"
# subject = "builtins.int.__rfloordiv__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rfloordiv__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rfloordiv__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rlshift____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rlshift____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rlshift____value_as_int_wrong"
# subject = "builtins.int.__rlshift__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rlshift__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rlshift__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rmod____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rmod____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rmod____value_as_int_wrong"
# subject = "builtins.int.__rmod__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rmod__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rmod__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rmul____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rmul____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rmul____value_as_int_wrong"
# subject = "builtins.int.__rmul__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rmul__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rmul__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____ror____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____ror____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____ror____value_as_int_wrong"
# subject = "builtins.int.__ror__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__ror__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__ror__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____round____ndigits_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____round____ndigits_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____round____ndigits_as_SupportsIndex_wrong"
# subject = "builtins.int.__round__(ndigits: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__round__(ndigits: SupportsIndex); call it with the wrong type.

typeshed contract: ndigits is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import int
obj = object.__new__(int)
try:
    obj.__round__(_W())  # ndigits: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____round____ndigits_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____round____ndigits_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____round____ndigits_as_typed_wrong"
# subject = "builtins.int.__round__(ndigits: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__round__(ndigits: typed); call it with the wrong type.

typeshed contract: ndigits is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import int
obj = object.__new__(int)
try:
    obj.__round__(_W())  # ndigits: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rpow____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rpow____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rpow____value_as_int_wrong"
# subject = "builtins.int.__rpow__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rpow__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rpow__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rrshift____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rrshift____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rrshift____value_as_int_wrong"
# subject = "builtins.int.__rrshift__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rrshift__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rrshift__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rshift____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rshift____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rshift____value_as_int_wrong"
# subject = "builtins.int.__rshift__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rshift__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rshift__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rsub____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rsub____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rsub____value_as_int_wrong"
# subject = "builtins.int.__rsub__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rsub__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rsub__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rtruediv____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rtruediv____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rtruediv____value_as_int_wrong"
# subject = "builtins.int.__rtruediv__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rtruediv__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rtruediv__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____rxor____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____rxor____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____rxor____value_as_int_wrong"
# subject = "builtins.int.__rxor__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__rxor__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__rxor__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____sub____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____sub____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____sub____value_as_int_wrong"
# subject = "builtins.int.__sub__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__sub__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__sub__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____truediv____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____truediv____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____truediv____value_as_int_wrong"
# subject = "builtins.int.__truediv__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__truediv__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__truediv__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int____xor____value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int____xor____value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int____xor____value_as_int_wrong"
# subject = "builtins.int.__xor__(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.__xor__(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import int
obj = object.__new__(int)
try:
    obj.__xor__("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int__from_bytes__bytes_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int__from_bytes__bytes_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int__from_bytes__bytes_as_typed_wrong"
# subject = "builtins.int.from_bytes(bytes: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.from_bytes(bytes: typed); call it with the wrong type.

typeshed contract: bytes is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import int
try:
    int.from_bytes(_W())  # bytes: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/int__to_bytes__length_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_int__to_bytes__length_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "int__to_bytes__length_as_SupportsIndex_wrong"
# subject = "builtins.int.to_bytes(length: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.int.to_bytes(length: SupportsIndex); call it with the wrong type.

typeshed contract: length is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import int
obj = object.__new__(int)
try:
    obj.to_bytes(_W())  # length: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/isinstance__class_or_tuple_as__ClassInfo_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_isinstance__class_or_tuple_as__ClassInfo_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "isinstance__class_or_tuple_as__ClassInfo_wrong"
# subject = "builtins.isinstance(class_or_tuple: _ClassInfo)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.isinstance(class_or_tuple: _ClassInfo); call it with the wrong type.

typeshed contract: class_or_tuple is _ClassInfo. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    isinstance(None, _W())  # class_or_tuple: _ClassInfo <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/isinstance__obj_rejects_keyword_argument.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_isinstance__obj_rejects_keyword_argument() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "isinstance__obj_rejects_keyword_argument"
# subject = "builtins.isinstance(obj, class_or_tuple, /)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.isinstance(obj, class_or_tuple, /) is positional-only
in CPython — calling it with keywords, `isinstance(obj=1, class_or_tuple=int)`,
raises `TypeError: isinstance() takes no keyword arguments` even though both
values are well-typed.

The generated TypeSpec contract retains `/` as `ParamSpecKind::PosOnly`, and
its structured binder is authoritative for call shape even though runtime
builtin registration also exposes a `Ty::Fn`. The companion case
`isinstance__class_or_tuple_as__ClassInfo_wrong.py` covers the argument type
contract; this fixture independently locks positional-only binding."""

try:
    isinstance(obj=1, class_or_tuple=int)  # isinstance is positional-only
    print("no_typeerror:")  # CPython accepted the keyword args; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/issubclass__cls_as_type_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_issubclass__cls_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "issubclass__cls_as_type_wrong"
# subject = "builtins.issubclass(cls: type)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.issubclass(cls: type); call it with the wrong type.

typeshed contract: cls is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    issubclass(_W(), None)  # cls: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/iter__object_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_iter__object_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "iter__object_as_Callable_wrong"
# subject = "builtins.iter(object: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.iter(object: Callable); call it with the wrong type.

typeshed contract: object is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    iter(_W(), None)  # object: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/iter__object_as_SupportsIter_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_iter__object_as_SupportsIter_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "iter__object_as_SupportsIter_wrong"
# subject = "builtins.iter(object: SupportsIter)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.iter(object: SupportsIter); call it with the wrong type.

typeshed contract: object is SupportsIter. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    iter(_W())  # object: SupportsIter <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/iter__object_as__GetItemIterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_iter__object_as__GetItemIterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "iter__object_as__GetItemIterable_wrong"
# subject = "builtins.iter(object: _GetItemIterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.iter(object: _GetItemIterable); call it with the wrong type.

typeshed contract: object is _GetItemIterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    iter(_W())  # object: _GetItemIterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/len__obj_as_Sized_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_len__obj_as_Sized_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "len__obj_as_Sized_wrong"
# subject = "builtins.len(obj: Sized)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.len(obj: Sized); call it with the wrong type.

typeshed contract: obj is Sized. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    len(_W())  # obj: Sized <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____add____value_as_list_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____add____value_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____add____value_as_list_wrong"
# subject = "builtins.list.__add__(value: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__add__(value: list); call it with the wrong type.

typeshed contract: value is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import list
obj = []
try:
    obj.__add__(12345)  # value: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____delitem____key_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____delitem____key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____delitem____key_as_typed_wrong"
# subject = "builtins.list.__delitem__(key: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__delitem__(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = []
try:
    obj.__delitem__(_W())  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____ge____value_as_list_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____ge____value_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____ge____value_as_list_wrong"
# subject = "builtins.list.__ge__(value: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__ge__(value: list); call it with the wrong type.

typeshed contract: value is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import list
obj = []
try:
    obj.__ge__(12345)  # value: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____getitem____i_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____getitem____i_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____getitem____i_as_SupportsIndex_wrong"
# subject = "builtins.list.__getitem__(i: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__getitem__(i: SupportsIndex); call it with the wrong type.

typeshed contract: i is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = []
try:
    obj.__getitem__(_W())  # i: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____getitem____s_as_slice_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____getitem____s_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____getitem____s_as_slice_wrong"
# subject = "builtins.list.__getitem__(s: slice)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__getitem__(s: slice); call it with the wrong type.

typeshed contract: s is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = []
try:
    obj.__getitem__(_W())  # s: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____gt____value_as_list_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____gt____value_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____gt____value_as_list_wrong"
# subject = "builtins.list.__gt__(value: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__gt__(value: list); call it with the wrong type.

typeshed contract: value is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import list
obj = []
try:
    obj.__gt__(12345)  # value: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____iadd____value_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____iadd____value_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____iadd____value_as_Iterable_wrong"
# subject = "builtins.list.__iadd__(value: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__iadd__(value: Iterable); call it with the wrong type.

typeshed contract: value is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = object.__new__(list)
try:
    obj.__iadd__(_W())  # value: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____imul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____imul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____imul____value_as_SupportsIndex_wrong"
# subject = "builtins.list.__imul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__imul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = object.__new__(list)
try:
    obj.__imul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____le____value_as_list_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____le____value_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____le____value_as_list_wrong"
# subject = "builtins.list.__le__(value: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__le__(value: list); call it with the wrong type.

typeshed contract: value is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import list
obj = []
try:
    obj.__le__(12345)  # value: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____lt____value_as_list_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____lt____value_as_list_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____lt____value_as_list_wrong"
# subject = "builtins.list.__lt__(value: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__lt__(value: list); call it with the wrong type.

typeshed contract: value is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import list
obj = []
try:
    obj.__lt__(12345)  # value: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____mul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____mul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____mul____value_as_SupportsIndex_wrong"
# subject = "builtins.list.__mul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__mul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = object.__new__(list)
try:
    obj.__mul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____rmul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____rmul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____rmul____value_as_SupportsIndex_wrong"
# subject = "builtins.list.__rmul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__rmul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = object.__new__(list)
try:
    obj.__rmul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____setitem____key_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____setitem____key_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____setitem____key_as_SupportsIndex_wrong"
# subject = "builtins.list.__setitem__(key: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__setitem__(key: SupportsIndex); call it with the wrong type.

typeshed contract: key is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = []
try:
    obj.__setitem__(_W(), None)  # key: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list____setitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list____setitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list____setitem____key_as_slice_wrong"
# subject = "builtins.list.__setitem__(key: slice)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__setitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = []
try:
    obj.__setitem__(_W(), None)  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list__append__object_as__T_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list__append__object_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list__append__object_as__T_wrong"
# subject = "builtins.list.append(object: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.append(object: _T); call it with the wrong type.

typeshed contract: object is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    def __eq__(self, other):
        return True


from builtins import list
obj: list[int] = [1]
try:
    obj.append(_W())  # object: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list__count__value_as__T_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list__count__value_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list__count__value_as__T_wrong"
# subject = "builtins.list.count(value: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.count(value: _T); call it with the wrong type.

typeshed contract: value is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    def __eq__(self, other):
        return True


from builtins import list
obj: list[int] = [1]
try:
    obj.count(_W())  # value: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list__extend__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list__extend__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list__extend__iterable_as_Iterable_wrong"
# subject = "builtins.list.extend(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.extend(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = object.__new__(list)
try:
    obj.extend(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list__index__value_as__T_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list__index__value_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list__index__value_as__T_wrong"
# subject = "builtins.list.index(value: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.index(value: _T); call it with the wrong type.

typeshed contract: value is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    def __eq__(self, other):
        return True


from builtins import list
obj: list[int] = [1]
try:
    obj.index(_W())  # value: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list__init__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list__init__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list__init__iterable_as_Iterable_wrong"
# subject = "builtins.list.__init__(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.__init__(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
try:
    list(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list__insert__index_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list__insert__index_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list__insert__index_as_SupportsIndex_wrong"
# subject = "builtins.list.insert(index: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.insert(index: SupportsIndex); call it with the wrong type.

typeshed contract: index is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = object.__new__(list)
try:
    obj.insert(_W(), None)  # index: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list__pop__index_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list__pop__index_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list__pop__index_as_SupportsIndex_wrong"
# subject = "builtins.list.pop(index: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.pop(index: SupportsIndex); call it with the wrong type.

typeshed contract: index is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import list
obj = object.__new__(list)
try:
    obj.pop(_W())  # index: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/list__remove__value_as__T_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_list__remove__value_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "list__remove__value_as__T_wrong"
# subject = "builtins.list.remove(value: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.list.remove(value: _T); call it with the wrong type.

typeshed contract: value is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    def __eq__(self, other):
        return True


from builtins import list
obj: list[int] = [1]
try:
    obj.remove(_W())  # value: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/map____new____func_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_map____new____func_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "map____new____func_as_Callable_wrong"
# subject = "builtins.map.__new__(func: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.map.__new__(func: Callable); call it with the wrong type.

typeshed contract: func is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import map
try:
    map.__new__(map, _W(), None)  # func: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview____buffer____flags_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview____buffer____flags_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview____buffer____flags_as_int_wrong"
# subject = "builtins.memoryview.__buffer__(flags: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.__buffer__(flags: int); call it with the wrong type.

typeshed contract: flags is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import memoryview
obj = object.__new__(memoryview)
try:
    obj.__buffer__("not_an_int")  # flags: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview____exit____exc_type_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview____exit____exc_type_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview____exit____exc_type_as_typed_wrong"
# subject = "builtins.memoryview.__exit__(exc_type: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.__exit__(exc_type: typed); call it with the wrong type.

typeshed contract: exc_type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import memoryview
obj = memoryview(b"abc")
try:
    obj.__exit__(_W(), None, None)  # exc_type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview____getitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview____getitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview____getitem____key_as_slice_wrong"
# subject = "builtins.memoryview.__getitem__(key: slice)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.__getitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import memoryview
obj = memoryview(b"abc")
try:
    obj.__getitem__(_W())  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview____getitem____key_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview____getitem____key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview____getitem____key_as_typed_wrong"
# subject = "builtins.memoryview.__getitem__(key: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.__getitem__(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import memoryview
obj = memoryview(b"abc")
try:
    obj.__getitem__(_W())  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview____new____obj_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview____new____obj_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview____new____obj_as_ReadableBuffer_wrong"
# subject = "builtins.memoryview.__new__(obj: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.__new__(obj: ReadableBuffer); call it with the wrong type.

typeshed contract: obj is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import memoryview
obj = object.__new__(memoryview)
try:
    obj.__new__(_W())  # obj: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview____release_buffer____buffer_as_memoryview_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview____release_buffer____buffer_as_memoryview_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview____release_buffer____buffer_as_memoryview_wrong"
# subject = "builtins.memoryview.__release_buffer__(buffer: memoryview)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.__release_buffer__(buffer: memoryview); call it with the wrong type.

typeshed contract: buffer is memoryview. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import memoryview
obj = memoryview(b"abc")
try:
    obj.__release_buffer__(12345)  # buffer: memoryview <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview____setitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview____setitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview____setitem____key_as_slice_wrong"
# subject = "builtins.memoryview.__setitem__(key: slice)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.__setitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import memoryview
obj = memoryview(bytearray(b"abc"))
try:
    obj.__setitem__(_W(), None)  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview____setitem____key_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview____setitem____key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview____setitem____key_as_typed_wrong"
# subject = "builtins.memoryview.__setitem__(key: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.__setitem__(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import memoryview
obj = memoryview(bytearray(b"abc"))
try:
    obj.__setitem__(_W(), None)  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview__cast__format_as_Literal_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview__cast__format_as_Literal_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview__cast__format_as_Literal_wrong"
# subject = "builtins.memoryview.cast(format: Literal)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.cast(format: Literal); call it with the wrong type.

typeshed contract: format is Literal. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import memoryview
obj = object.__new__(memoryview)
try:
    obj.cast(_W())  # format: Literal <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview__cast__format_as__IntegerFormats_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview__cast__format_as__IntegerFormats_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview__cast__format_as__IntegerFormats_wrong"
# subject = "builtins.memoryview.cast(format: _IntegerFormats)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.cast(format: _IntegerFormats); call it with the wrong type.

typeshed contract: format is _IntegerFormats. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import memoryview
obj = object.__new__(memoryview)
try:
    obj.cast(_W())  # format: _IntegerFormats <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview__hex__sep_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview__hex__sep_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview__hex__sep_as_typed_wrong"
# subject = "builtins.memoryview.hex(sep: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.hex(sep: typed); call it with the wrong type.

typeshed contract: sep is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import memoryview
obj = object.__new__(memoryview)
try:
    obj.hex(_W())  # sep: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview__index__start_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview__index__start_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview__index__start_as_SupportsIndex_wrong"
# subject = "builtins.memoryview.index(start: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.index(start: SupportsIndex); call it with the wrong type.

typeshed contract: start is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import memoryview
obj = object.__new__(memoryview)
try:
    obj.index(None, _W())  # start: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/memoryview__tobytes__order_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_memoryview__tobytes__order_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "memoryview__tobytes__order_as_typed_wrong"
# subject = "builtins.memoryview.tobytes(order: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.memoryview.tobytes(order: typed); call it with the wrong type.

typeshed contract: order is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import memoryview
obj = memoryview(b"abc")
try:
    obj.tobytes(_W())  # order: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/next__i_as_SupportsNext_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_next__i_as_SupportsNext_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "next__i_as_SupportsNext_wrong"
# subject = "builtins.next(i: SupportsNext)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.next(i: SupportsNext); call it with the wrong type.

typeshed contract: i is SupportsNext. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    next(_W())  # i: SupportsNext <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/object____delattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_object____delattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "object____delattr____name_as_str_wrong"
# subject = "builtins.object.__delattr__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.object.__delattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import object
obj = object.__new__(object)
try:
    obj.__delattr__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/object____format____format_spec_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_object____format____format_spec_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "object____format____format_spec_as_str_wrong"
# subject = "builtins.object.__format__(format_spec: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.object.__format__(format_spec: str); call it with the wrong type.

typeshed contract: format_spec is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import object
obj = object.__new__(object)
try:
    obj.__format__(12345)  # format_spec: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/object____getattribute____name_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_object____getattribute____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "object____getattribute____name_as_str_wrong"
# subject = "builtins.object.__getattribute__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.object.__getattribute__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import object
obj = object.__new__(object)
try:
    obj.__getattribute__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/object____reduce_ex____protocol_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_object____reduce_ex____protocol_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "object____reduce_ex____protocol_as_SupportsIndex_wrong"
# subject = "builtins.object.__reduce_ex__(protocol: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.object.__reduce_ex__(protocol: SupportsIndex); call it with the wrong type.

typeshed contract: protocol is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import object
obj = object.__new__(object)
try:
    obj.__reduce_ex__(_W())  # protocol: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/object____setattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_object____setattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "object____setattr____name_as_str_wrong"
# subject = "builtins.object.__setattr__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.object.__setattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import object
obj = object.__new__(object)
try:
    obj.__setattr__(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/object____subclasshook____subclass_as_type_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_object____subclasshook____subclass_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "object____subclasshook____subclass_as_type_wrong"
# subject = "builtins.object.__subclasshook__(subclass: type)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.object.__subclasshook__(subclass: type); call it with the wrong type.

typeshed contract: subclass is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import object
try:
    object.__subclasshook__(_W())  # subclass: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/oct__number_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_oct__number_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "oct__number_as_SupportsIndex_wrong"
# subject = "builtins.oct(number: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.oct(number: SupportsIndex); call it with the wrong type.

typeshed contract: number is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    oct(_W())  # number: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/open__file_as_FileDescriptorOrPath_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_open__file_as_FileDescriptorOrPath_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "open__file_as_FileDescriptorOrPath_wrong"
# subject = "builtins.open(file: FileDescriptorOrPath)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.open(file: FileDescriptorOrPath); call it with the wrong type.

typeshed contract: file is FileDescriptorOrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    open(_W())  # file: FileDescriptorOrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/ord__c_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_ord__c_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "ord__c_as_typed_wrong"
# subject = "builtins.ord(c: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.ord(c: typed); call it with the wrong type.

typeshed contract: c is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    ord(_W())  # c: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/pow__base_as__SupportsPow2_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_pow__base_as__SupportsPow2_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "pow__base_as__SupportsPow2_wrong"
# subject = "builtins.pow(base: _SupportsPow2)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.pow(base: _SupportsPow2); call it with the wrong type.

typeshed contract: base is _SupportsPow2. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    pow(_W(), None)  # base: _SupportsPow2 <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/pow__base_as__SupportsPow3NoneOnly_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_pow__base_as__SupportsPow3NoneOnly_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "pow__base_as__SupportsPow3NoneOnly_wrong"
# subject = "builtins.pow(base: _SupportsPow3NoneOnly)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.pow(base: _SupportsPow3NoneOnly); call it with the wrong type.

typeshed contract: base is _SupportsPow3NoneOnly. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    pow(_W(), None)  # base: _SupportsPow3NoneOnly <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/pow__base_as__SupportsPow3_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_pow__base_as__SupportsPow3_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "pow__base_as__SupportsPow3_wrong"
# subject = "builtins.pow(base: _SupportsPow3)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.pow(base: _SupportsPow3); call it with the wrong type.

typeshed contract: base is _SupportsPow3. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    pow(_W(), None, None)  # base: _SupportsPow3 <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/property____get____owner_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_property____get____owner_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "property____get____owner_as_typed_wrong"
# subject = "builtins.property.__get__(owner: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.property.__get__(owner: typed); call it with the wrong type.

typeshed contract: owner is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import property
def _f(self=None):
    return None


obj = property(_f)
try:
    obj.__get__(None, _W())  # owner: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/property__deleter__fdel_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_property__deleter__fdel_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "property__deleter__fdel_as_Callable_wrong"
# subject = "builtins.property.deleter(fdel: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.property.deleter(fdel: Callable); call it with the wrong type.

typeshed contract: fdel is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import property
def _f(self=None):
    return None


obj = property(_f)
try:
    obj.deleter(_W())  # fdel: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/property__getter__fget_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_property__getter__fget_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "property__getter__fget_as_Callable_wrong"
# subject = "builtins.property.getter(fget: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.property.getter(fget: Callable); call it with the wrong type.

typeshed contract: fget is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import property
def _f(self=None):
    return None


obj = property(_f)
try:
    obj.getter(_W())  # fget: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/property__init__fget_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_property__init__fget_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "property__init__fget_as_typed_wrong"
# subject = "builtins.property.__init__(fget: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.property.__init__(fget: typed); call it with the wrong type.

typeshed contract: fget is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import property
try:
    property(_W())  # fget: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/property__setter__fset_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_property__setter__fset_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "property__setter__fset_as_Callable_wrong"
# subject = "builtins.property.setter(fset: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.property.setter(fset: Callable); call it with the wrong type.

typeshed contract: fset is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import property
def _f(self=None):
    return None


obj = property(_f)
try:
    obj.setter(_W())  # fset: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/range____getitem____key_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_range____getitem____key_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "range____getitem____key_as_SupportsIndex_wrong"
# subject = "builtins.range.__getitem__(key: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.range.__getitem__(key: SupportsIndex); call it with the wrong type.

typeshed contract: key is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import range
obj = range(3)
try:
    obj.__getitem__(_W())  # key: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/range____getitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_range____getitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "range____getitem____key_as_slice_wrong"
# subject = "builtins.range.__getitem__(key: slice)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.range.__getitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import range
obj = range(3)
try:
    obj.__getitem__(_W())  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/range____new____start_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_range____new____start_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "range____new____start_as_SupportsIndex_wrong"
# subject = "builtins.range.__new__(start: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.range.__new__(start: SupportsIndex); call it with the wrong type.

typeshed contract: start is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import range
try:
    range.__new__(range, _W(), 3)  # start: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/range____new____stop_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_range____new____stop_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "range____new____stop_as_SupportsIndex_wrong"
# subject = "builtins.range.__new__(stop: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.range.__new__(stop: SupportsIndex); call it with the wrong type.

typeshed contract: stop is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import range
try:
    range.__new__(range, _W())  # stop: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/range__count__value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_range__count__value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "range__count__value_as_int_wrong"
# subject = "builtins.range.count(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.range.count(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import range
obj = object.__new__(range)
try:
    obj.count("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/range__index__value_as_int_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_range__index__value_as_int_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "range__index__value_as_int_wrong"
# subject = "builtins.range.index(value: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.range.index(value: int); call it with the wrong type.

typeshed contract: value is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import range
obj = object.__new__(range)
try:
    obj.index("not_an_int")  # value: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/reversed____new____sequence_as_Reversible_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_reversed____new____sequence_as_Reversible_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "reversed____new____sequence_as_Reversible_wrong"
# subject = "builtins.reversed.__new__(sequence: Reversible)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.reversed.__new__(sequence: Reversible); call it with the wrong type.

typeshed contract: sequence is Reversible. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import reversed
try:
    reversed.__new__(reversed, _W())  # sequence: Reversible <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/reversed____new____sequence_as_SupportsLenAndGetItem_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_reversed____new____sequence_as_SupportsLenAndGetItem_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "reversed____new____sequence_as_SupportsLenAndGetItem_wrong"
# subject = "builtins.reversed.__new__(sequence: SupportsLenAndGetItem)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.reversed.__new__(sequence: SupportsLenAndGetItem); call it with the wrong type.

typeshed contract: sequence is SupportsLenAndGetItem. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import reversed
try:
    reversed.__new__(reversed, _W())  # sequence: SupportsLenAndGetItem <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/round__number_as__SupportsRound1_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_round__number_as__SupportsRound1_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "round__number_as__SupportsRound1_wrong"
# subject = "builtins.round(number: _SupportsRound1)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.round(number: _SupportsRound1); call it with the wrong type.

typeshed contract: number is _SupportsRound1. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    round(_W())  # number: _SupportsRound1 <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/round__number_as__SupportsRound2_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_round__number_as__SupportsRound2_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "round__number_as__SupportsRound2_wrong"
# subject = "builtins.round(number: _SupportsRound2)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.round(number: _SupportsRound2); call it with the wrong type.

typeshed contract: number is _SupportsRound2. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    round(_W(), None)  # number: _SupportsRound2 <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/sentinel____new____name_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_sentinel____new____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "sentinel____new____name_as_str_wrong"
# subject = "builtins.sentinel.__new__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.sentinel.__new__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import sentinel
obj = object.__new__(sentinel)
try:
    obj.__new__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____and____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____and____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____and____value_as_AbstractSet_wrong"
# subject = "builtins.set.__and__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__and__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__and__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____ge____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____ge____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____ge____value_as_AbstractSet_wrong"
# subject = "builtins.set.__ge__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__ge__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__ge__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____gt____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____gt____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____gt____value_as_AbstractSet_wrong"
# subject = "builtins.set.__gt__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__gt__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__gt__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____iand____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____iand____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____iand____value_as_AbstractSet_wrong"
# subject = "builtins.set.__iand__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__iand__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__iand__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____ior____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____ior____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____ior____value_as_AbstractSet_wrong"
# subject = "builtins.set.__ior__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__ior__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__ior__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____isub____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____isub____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____isub____value_as_AbstractSet_wrong"
# subject = "builtins.set.__isub__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__isub__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__isub__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____ixor____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____ixor____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____ixor____value_as_AbstractSet_wrong"
# subject = "builtins.set.__ixor__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__ixor__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__ixor__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____le____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____le____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____le____value_as_AbstractSet_wrong"
# subject = "builtins.set.__le__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__le__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__le__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____lt____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____lt____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____lt____value_as_AbstractSet_wrong"
# subject = "builtins.set.__lt__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__lt__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__lt__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____or____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____or____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____or____value_as_AbstractSet_wrong"
# subject = "builtins.set.__or__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__or__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__or__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____sub____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____sub____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____sub____value_as_AbstractSet_wrong"
# subject = "builtins.set.__sub__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__sub__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__sub__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set____xor____value_as_AbstractSet_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set____xor____value_as_AbstractSet_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set____xor____value_as_AbstractSet_wrong"
# subject = "builtins.set.__xor__(value: AbstractSet)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__xor__(value: AbstractSet); call it with the wrong type.

typeshed contract: value is AbstractSet. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = set()
try:
    obj.__xor__(_W())  # value: AbstractSet <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set__add__element_as__T_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set__add__element_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set__add__element_as__T_wrong"
# subject = "builtins.set.add(element: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.add(element: _T); call it with the wrong type.

typeshed contract: element is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    def __hash__(self):
        return hash(1)

    def __eq__(self, other):
        return True


from builtins import set
obj: set[int] = {1}
try:
    obj.add(_W())  # element: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set__init__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set__init__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set__init__iterable_as_Iterable_wrong"
# subject = "builtins.set.__init__(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.__init__(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
try:
    set(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set__isdisjoint__s_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set__isdisjoint__s_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set__isdisjoint__s_as_Iterable_wrong"
# subject = "builtins.set.isdisjoint(s: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.isdisjoint(s: Iterable); call it with the wrong type.

typeshed contract: s is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = object.__new__(set)
try:
    obj.isdisjoint(_W())  # s: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set__issubset__s_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set__issubset__s_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set__issubset__s_as_Iterable_wrong"
# subject = "builtins.set.issubset(s: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.issubset(s: Iterable); call it with the wrong type.

typeshed contract: s is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = object.__new__(set)
try:
    obj.issubset(_W())  # s: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set__issuperset__s_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set__issuperset__s_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set__issuperset__s_as_Iterable_wrong"
# subject = "builtins.set.issuperset(s: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.issuperset(s: Iterable); call it with the wrong type.

typeshed contract: s is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = object.__new__(set)
try:
    obj.issuperset(_W())  # s: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set__remove__element_as__T_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set__remove__element_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set__remove__element_as__T_wrong"
# subject = "builtins.set.remove(element: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.remove(element: _T); call it with the wrong type.

typeshed contract: element is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    def __hash__(self):
        return hash(1)

    def __eq__(self, other):
        return True


from builtins import set
obj: set[int] = {1}
try:
    obj.remove(_W())  # element: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set__symmetric_difference__s_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set__symmetric_difference__s_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set__symmetric_difference__s_as_Iterable_wrong"
# subject = "builtins.set.symmetric_difference(s: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.symmetric_difference(s: Iterable); call it with the wrong type.

typeshed contract: s is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = object.__new__(set)
try:
    obj.symmetric_difference(_W())  # s: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/set__symmetric_difference_update__s_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_set__symmetric_difference_update__s_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "set__symmetric_difference_update__s_as_Iterable_wrong"
# subject = "builtins.set.symmetric_difference_update(s: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.set.symmetric_difference_update(s: Iterable); call it with the wrong type.

typeshed contract: s is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import set
obj = object.__new__(set)
try:
    obj.symmetric_difference_update(_W())  # s: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/setattr__name_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_setattr__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "setattr__name_as_str_wrong"
# subject = "builtins.setattr(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.setattr(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

try:
    setattr(None, 12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/slice__indices__len_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_slice__indices__len_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "slice__indices__len_as_SupportsIndex_wrong"
# subject = "builtins.slice.indices(len: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.slice.indices(len: SupportsIndex); call it with the wrong type.

typeshed contract: len is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import slice
obj = object.__new__(slice)
try:
    obj.indices(_W())  # len: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/sorted__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_sorted__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "sorted__iterable_as_Iterable_wrong"
# subject = "builtins.sorted(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.sorted(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


try:
    sorted(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/staticmethod____get____instance_as__T_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_staticmethod____get____instance_as__T_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "staticmethod____get____instance_as__T_wrong"
# subject = "builtins.staticmethod.__get__(instance: _T)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.staticmethod.__get__(instance: _T); call it with the wrong type.

typeshed contract: instance is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import staticmethod
obj = staticmethod(lambda: None)
try:
    obj.__get__(_W())  # instance: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/staticmethod____get____instance_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_staticmethod____get____instance_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "staticmethod____get____instance_as_typed_wrong"
# subject = "builtins.staticmethod.__get__(instance: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.staticmethod.__get__(instance: typed); call it with the wrong type.

typeshed contract: instance is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import staticmethod
obj = staticmethod(lambda: None)
try:
    obj.__get__(_W(), None)  # instance: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/staticmethod__init__f_as_Callable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_staticmethod__init__f_as_Callable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "staticmethod__init__f_as_Callable_wrong"
# subject = "builtins.staticmethod.__init__(f: Callable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.staticmethod.__init__(f: Callable); call it with the wrong type.

typeshed contract: f is Callable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import staticmethod
try:
    staticmethod(_W())  # f: Callable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____add____value_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____add____value_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____add____value_as_LiteralString_wrong"
# subject = "builtins.str.__add__(value: LiteralString)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__add__(value: LiteralString); call it with the wrong type.

typeshed contract: value is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.__add__(_W())  # value: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____add____value_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____add____value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____add____value_as_str_wrong"
# subject = "builtins.str.__add__(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__add__(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.__add__(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____contains____key_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____contains____key_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____contains____key_as_str_wrong"
# subject = "builtins.str.__contains__(key: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__contains__(key: str); call it with the wrong type.

typeshed contract: key is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.__contains__(12345)  # key: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____format____format_spec_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____format____format_spec_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____format____format_spec_as_str_wrong"
# subject = "builtins.str.__format__(format_spec: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__format__(format_spec: str); call it with the wrong type.

typeshed contract: format_spec is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.__format__(12345)  # format_spec: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____ge____value_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____ge____value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____ge____value_as_str_wrong"
# subject = "builtins.str.__ge__(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__ge__(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.__ge__(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____getitem____key_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____getitem____key_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____getitem____key_as_typed_wrong"
# subject = "builtins.str.__getitem__(key: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__getitem__(key: typed); call it with the wrong type.

typeshed contract: key is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.__getitem__(_W())  # key: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____gt____value_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____gt____value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____gt____value_as_str_wrong"
# subject = "builtins.str.__gt__(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__gt__(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.__gt__(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____le____value_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____le____value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____le____value_as_str_wrong"
# subject = "builtins.str.__le__(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__le__(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.__le__(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____lt____value_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____lt____value_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____lt____value_as_str_wrong"
# subject = "builtins.str.__lt__(value: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__lt__(value: str); call it with the wrong type.

typeshed contract: value is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.__lt__(12345)  # value: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____mul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____mul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____mul____value_as_SupportsIndex_wrong"
# subject = "builtins.str.__mul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__mul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.__mul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____new____object_as_ReadableBuffer_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____new____object_as_ReadableBuffer_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____new____object_as_ReadableBuffer_wrong"
# subject = "builtins.str.__new__(object: ReadableBuffer)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__new__(object: ReadableBuffer); call it with the wrong type.

typeshed contract: object is ReadableBuffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
try:
    str.__new__(str, _W(), "utf-8")  # object: ReadableBuffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str____rmul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str____rmul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str____rmul____value_as_SupportsIndex_wrong"
# subject = "builtins.str.__rmul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.__rmul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.__rmul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__center__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__center__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__center__width_as_SupportsIndex_wrong"
# subject = "builtins.str.center(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.center(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.center(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__count__sub_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__count__sub_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__count__sub_as_str_wrong"
# subject = "builtins.str.count(sub: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.count(sub: str); call it with the wrong type.

typeshed contract: sub is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.count(12345)  # sub: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__encode__encoding_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__encode__encoding_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__encode__encoding_as_str_wrong"
# subject = "builtins.str.encode(encoding: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.encode(encoding: str); call it with the wrong type.

typeshed contract: encoding is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.encode(12345)  # encoding: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__endswith__suffix_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__endswith__suffix_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__endswith__suffix_as_typed_wrong"
# subject = "builtins.str.endswith(suffix: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.endswith(suffix: typed); call it with the wrong type.

typeshed contract: suffix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.endswith(_W())  # suffix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__expandtabs__tabsize_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__expandtabs__tabsize_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__expandtabs__tabsize_as_SupportsIndex_wrong"
# subject = "builtins.str.expandtabs(tabsize: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.expandtabs(tabsize: SupportsIndex); call it with the wrong type.

typeshed contract: tabsize is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.expandtabs(_W())  # tabsize: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__find__sub_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__find__sub_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__find__sub_as_str_wrong"
# subject = "builtins.str.find(sub: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.find(sub: str); call it with the wrong type.

typeshed contract: sub is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.find(12345)  # sub: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__format_map__mapping_as__FormatMapMapping_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__format_map__mapping_as__FormatMapMapping_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__format_map__mapping_as__FormatMapMapping_wrong"
# subject = "builtins.str.format_map(mapping: _FormatMapMapping)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.format_map(mapping: _FormatMapMapping); call it with the wrong type.

typeshed contract: mapping is _FormatMapMapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.format_map(_W())  # mapping: _FormatMapMapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__index__sub_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__index__sub_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__index__sub_as_str_wrong"
# subject = "builtins.str.index(sub: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.index(sub: str); call it with the wrong type.

typeshed contract: sub is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.index(12345)  # sub: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__join__iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__join__iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__join__iterable_as_Iterable_wrong"
# subject = "builtins.str.join(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.join(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.join(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__ljust__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__ljust__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__ljust__width_as_SupportsIndex_wrong"
# subject = "builtins.str.ljust(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.ljust(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.ljust(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__lstrip__chars_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__lstrip__chars_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__lstrip__chars_as_typed_wrong"
# subject = "builtins.str.lstrip(chars: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.lstrip(chars: typed); call it with the wrong type.

typeshed contract: chars is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.lstrip(_W())  # chars: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__maketrans__x_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__maketrans__x_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__maketrans__x_as_str_wrong"
# subject = "builtins.str.maketrans(x: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.maketrans(x: str); call it with the wrong type.

typeshed contract: x is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
try:
    str.maketrans(12345, "")  # x: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__maketrans__x_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__maketrans__x_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__maketrans__x_as_typed_wrong"
# subject = "builtins.str.maketrans(x: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.maketrans(x: typed); call it with the wrong type.

typeshed contract: x is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
try:
    str.maketrans(_W())  # x: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__partition__sep_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__partition__sep_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__partition__sep_as_LiteralString_wrong"
# subject = "builtins.str.partition(sep: LiteralString)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.partition(sep: LiteralString); call it with the wrong type.

typeshed contract: sep is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.partition(_W())  # sep: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__partition__sep_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__partition__sep_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__partition__sep_as_str_wrong"
# subject = "builtins.str.partition(sep: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.partition(sep: str); call it with the wrong type.

typeshed contract: sep is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.partition(12345)  # sep: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__removeprefix__prefix_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__removeprefix__prefix_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__removeprefix__prefix_as_LiteralString_wrong"
# subject = "builtins.str.removeprefix(prefix: LiteralString)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.removeprefix(prefix: LiteralString); call it with the wrong type.

typeshed contract: prefix is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.removeprefix(_W())  # prefix: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__removeprefix__prefix_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__removeprefix__prefix_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__removeprefix__prefix_as_str_wrong"
# subject = "builtins.str.removeprefix(prefix: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.removeprefix(prefix: str); call it with the wrong type.

typeshed contract: prefix is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.removeprefix(12345)  # prefix: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__removesuffix__suffix_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__removesuffix__suffix_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__removesuffix__suffix_as_LiteralString_wrong"
# subject = "builtins.str.removesuffix(suffix: LiteralString)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.removesuffix(suffix: LiteralString); call it with the wrong type.

typeshed contract: suffix is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.removesuffix(_W())  # suffix: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__removesuffix__suffix_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__removesuffix__suffix_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__removesuffix__suffix_as_str_wrong"
# subject = "builtins.str.removesuffix(suffix: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.removesuffix(suffix: str); call it with the wrong type.

typeshed contract: suffix is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.removesuffix(12345)  # suffix: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__replace__old_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__replace__old_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__replace__old_as_LiteralString_wrong"
# subject = "builtins.str.replace(old: LiteralString)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.replace(old: LiteralString); call it with the wrong type.

typeshed contract: old is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.replace(_W(), None)  # old: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__replace__old_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__replace__old_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__replace__old_as_str_wrong"
# subject = "builtins.str.replace(old: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.replace(old: str); call it with the wrong type.

typeshed contract: old is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.replace(12345, "")  # old: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__rfind__sub_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__rfind__sub_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__rfind__sub_as_str_wrong"
# subject = "builtins.str.rfind(sub: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.rfind(sub: str); call it with the wrong type.

typeshed contract: sub is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.rfind(12345)  # sub: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__rindex__sub_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__rindex__sub_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__rindex__sub_as_str_wrong"
# subject = "builtins.str.rindex(sub: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.rindex(sub: str); call it with the wrong type.

typeshed contract: sub is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.rindex(12345)  # sub: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__rjust__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__rjust__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__rjust__width_as_SupportsIndex_wrong"
# subject = "builtins.str.rjust(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.rjust(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.rjust(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__rpartition__sep_as_LiteralString_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__rpartition__sep_as_LiteralString_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__rpartition__sep_as_LiteralString_wrong"
# subject = "builtins.str.rpartition(sep: LiteralString)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.rpartition(sep: LiteralString); call it with the wrong type.

typeshed contract: sep is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.rpartition(_W())  # sep: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__rpartition__sep_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__rpartition__sep_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__rpartition__sep_as_str_wrong"
# subject = "builtins.str.rpartition(sep: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.rpartition(sep: str); call it with the wrong type.

typeshed contract: sep is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.rpartition(12345)  # sep: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__rsplit__sep_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__rsplit__sep_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__rsplit__sep_as_typed_wrong"
# subject = "builtins.str.rsplit(sep: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.rsplit(sep: typed); call it with the wrong type.

typeshed contract: sep is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.rsplit(_W())  # sep: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__rstrip__chars_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__rstrip__chars_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__rstrip__chars_as_typed_wrong"
# subject = "builtins.str.rstrip(chars: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.rstrip(chars: typed); call it with the wrong type.

typeshed contract: chars is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.rstrip(_W())  # chars: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__split__sep_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__split__sep_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__split__sep_as_typed_wrong"
# subject = "builtins.str.split(sep: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.split(sep: typed); call it with the wrong type.

typeshed contract: sep is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.split(_W())  # sep: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__splitlines__keepends_as_bool_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__splitlines__keepends_as_bool_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__splitlines__keepends_as_bool_wrong"
# subject = "builtins.str.splitlines(keepends: bool)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.splitlines(keepends: bool); call it with the wrong type.

typeshed contract: keepends is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import str
obj = str.__new__(str)
try:
    obj.splitlines("not_a_bool")  # keepends: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__startswith__prefix_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__startswith__prefix_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__startswith__prefix_as_typed_wrong"
# subject = "builtins.str.startswith(prefix: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.startswith(prefix: typed); call it with the wrong type.

typeshed contract: prefix is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.startswith(_W())  # prefix: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__strip__chars_as_typed_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__strip__chars_as_typed_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__strip__chars_as_typed_wrong"
# subject = "builtins.str.strip(chars: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.strip(chars: typed); call it with the wrong type.

typeshed contract: chars is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.strip(_W())  # chars: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__translate__table_as__TranslateTable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__translate__table_as__TranslateTable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__translate__table_as__TranslateTable_wrong"
# subject = "builtins.str.translate(table: _TranslateTable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.translate(table: _TranslateTable); call it with the wrong type.

typeshed contract: table is _TranslateTable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.translate(_W())  # table: _TranslateTable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/str__zfill__width_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_str__zfill__width_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "str__zfill__width_as_SupportsIndex_wrong"
# subject = "builtins.str.zfill(width: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.str.zfill(width: SupportsIndex); call it with the wrong type.

typeshed contract: width is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import str
obj = str.__new__(str)
try:
    obj.zfill(_W())  # width: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/tuple____add____value_as_tuple_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_tuple____add____value_as_tuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "tuple____add____value_as_tuple_wrong"
# subject = "builtins.tuple.__add__(value: tuple)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.tuple.__add__(value: tuple); call it with the wrong type.

typeshed contract: value is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import tuple
obj = object.__new__(tuple)
try:
    obj.__add__(12345)  # value: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/tuple____ge____value_as_tuple_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_tuple____ge____value_as_tuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "tuple____ge____value_as_tuple_wrong"
# subject = "builtins.tuple.__ge__(value: tuple)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.tuple.__ge__(value: tuple); call it with the wrong type.

typeshed contract: value is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import tuple
obj = object.__new__(tuple)
try:
    obj.__ge__(12345)  # value: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/tuple____getitem____key_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_tuple____getitem____key_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "tuple____getitem____key_as_SupportsIndex_wrong"
# subject = "builtins.tuple.__getitem__(key: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.tuple.__getitem__(key: SupportsIndex); call it with the wrong type.

typeshed contract: key is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import tuple
obj = object.__new__(tuple)
try:
    obj.__getitem__(_W())  # key: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/tuple____getitem____key_as_slice_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_tuple____getitem____key_as_slice_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "tuple____getitem____key_as_slice_wrong"
# subject = "builtins.tuple.__getitem__(key: slice)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.tuple.__getitem__(key: slice); call it with the wrong type.

typeshed contract: key is slice. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import tuple
obj = object.__new__(tuple)
try:
    obj.__getitem__(_W())  # key: slice <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/tuple____gt____value_as_tuple_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_tuple____gt____value_as_tuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "tuple____gt____value_as_tuple_wrong"
# subject = "builtins.tuple.__gt__(value: tuple)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.tuple.__gt__(value: tuple); call it with the wrong type.

typeshed contract: value is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import tuple
obj = object.__new__(tuple)
try:
    obj.__gt__(12345)  # value: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/tuple____le____value_as_tuple_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_tuple____le____value_as_tuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "tuple____le____value_as_tuple_wrong"
# subject = "builtins.tuple.__le__(value: tuple)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.tuple.__le__(value: tuple); call it with the wrong type.

typeshed contract: value is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import tuple
obj = object.__new__(tuple)
try:
    obj.__le__(12345)  # value: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/tuple____lt____value_as_tuple_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_tuple____lt____value_as_tuple_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "tuple____lt____value_as_tuple_wrong"
# subject = "builtins.tuple.__lt__(value: tuple)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.tuple.__lt__(value: tuple); call it with the wrong type.

typeshed contract: value is tuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import tuple
obj = object.__new__(tuple)
try:
    obj.__lt__(12345)  # value: tuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/tuple____mul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_tuple____mul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "tuple____mul____value_as_SupportsIndex_wrong"
# subject = "builtins.tuple.__mul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.tuple.__mul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import tuple
obj = object.__new__(tuple)
try:
    obj.__mul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/tuple____new____iterable_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_tuple____new____iterable_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "tuple____new____iterable_as_Iterable_wrong"
# subject = "builtins.tuple.__new__(iterable: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.tuple.__new__(iterable: Iterable); call it with the wrong type.

typeshed contract: iterable is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import tuple
obj = object.__new__(tuple)
try:
    obj.__new__(_W())  # iterable: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/tuple____rmul____value_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_tuple____rmul____value_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "tuple____rmul____value_as_SupportsIndex_wrong"
# subject = "builtins.tuple.__rmul__(value: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.tuple.__rmul__(value: SupportsIndex); call it with the wrong type.

typeshed contract: value is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import tuple
obj = object.__new__(tuple)
try:
    obj.__rmul__(_W())  # value: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/tuple__index__start_as_SupportsIndex_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_tuple__index__start_as_SupportsIndex_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "tuple__index__start_as_SupportsIndex_wrong"
# subject = "builtins.tuple.index(start: SupportsIndex)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.tuple.index(start: SupportsIndex); call it with the wrong type.

typeshed contract: start is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import tuple
obj = object.__new__(tuple)
try:
    obj.index(None, _W())  # start: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/type____new____name_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_type____new____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "type____new____name_as_str_wrong"
# subject = "builtins.type.__new__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.type.__new__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import type
obj = object.__new__(type)
try:
    obj.__new__(12345, None, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/type____prepare____name_as_str_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_type____prepare____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "type____prepare____name_as_str_wrong"
# subject = "builtins.type.__prepare__(name: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.type.__prepare__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from builtins import type
try:
    type.__prepare__(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/type____subclasscheck____subclass_as_type_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_type____subclasscheck____subclass_as_type_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "type____subclasscheck____subclass_as_type_wrong"
# subject = "builtins.type.__subclasscheck__(subclass: type)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.type.__subclasscheck__(subclass: type); call it with the wrong type.

typeshed contract: subclass is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import type
obj = object.__new__(type)
try:
    obj.__subclasscheck__(_W())  # subclass: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/builtin-libs/builtins/zip____new____iter1_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_builtin_libs_builtins_zip____new____iter1_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "builtins"
# dimension = "type"
# case = "zip____new____iter1_as_Iterable_wrong"
# subject = "builtins.zip.__new__(iter1: Iterable)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/builtins.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: builtins.zip.__new__(iter1: Iterable); call it with the wrong type.

typeshed contract: iter1 is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from builtins import zip
obj = object.__new__(zip)
try:
    obj.__new__(_W())  # iter1: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
