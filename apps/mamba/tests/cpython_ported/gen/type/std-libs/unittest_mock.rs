use super::super::super::super::harness::*;

/// Ported from `tests/cpython/type/std-libs/unittest_mock/AsyncMockMixin__assert_has_awaits__calls_as_Iterable_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_mock_AsyncMockMixin__assert_has_awaits__calls_as_Iterable_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "type"
# case = "AsyncMockMixin__assert_has_awaits__calls_as_Iterable_wrong"
# subject = "unittest.mock.AsyncMockMixin.assert_has_awaits(calls: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/mock.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.mock.AsyncMockMixin.assert_has_awaits(calls: Iterable); call it with the wrong type.

typeshed contract: calls is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.mock import AsyncMockMixin
obj = object.__new__(AsyncMockMixin)
try:
    obj.assert_has_awaits(_W())  # calls: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_mock/MagicProxy__init__name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_mock_MagicProxy__init__name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "type"
# case = "MagicProxy__init__name_as_str_wrong"
# subject = "unittest.mock.MagicProxy.__init__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/mock.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.mock.MagicProxy.__init__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.mock import MagicProxy
try:
    MagicProxy(12345, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_mock/NonCallableMock____delattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_mock_NonCallableMock____delattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "type"
# case = "NonCallableMock____delattr____name_as_str_wrong"
# subject = "unittest.mock.NonCallableMock.__delattr__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/mock.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.mock.NonCallableMock.__delattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.mock import NonCallableMock
obj = object.__new__(NonCallableMock)
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

/// Ported from `tests/cpython/type/std-libs/unittest_mock/NonCallableMock____getattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_mock_NonCallableMock____getattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "type"
# case = "NonCallableMock____getattr____name_as_str_wrong"
# subject = "unittest.mock.NonCallableMock.__getattr__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/mock.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.mock.NonCallableMock.__getattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.mock import NonCallableMock
obj = object.__new__(NonCallableMock)
try:
    obj.__getattr__(12345)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}

/// Ported from `tests/cpython/type/std-libs/unittest_mock/NonCallableMock____setattr____name_as_str_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_mock_NonCallableMock____setattr____name_as_str_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "type"
# case = "NonCallableMock____setattr____name_as_str_wrong"
# subject = "unittest.mock.NonCallableMock.__setattr__(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/mock.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.mock.NonCallableMock.__setattr__(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from unittest.mock import NonCallableMock
obj = object.__new__(NonCallableMock)
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

/// Ported from `tests/cpython/type/std-libs/unittest_mock/NonCallableMock__attach_mock__mock_as_NonCallableMock_wrong.py`.
#[test]
fn test_gen_type_std_libs_unittest_mock_NonCallableMock__attach_mock__mock_as_NonCallableMock_wrong() {
    let out = run_type_wall_fixture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unittest_mock"
# dimension = "type"
# case = "NonCallableMock__attach_mock__mock_as_NonCallableMock_wrong"
# subject = "unittest.mock.NonCallableMock.attach_mock(mock: NonCallableMock)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/unittest/mock.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: unittest.mock.NonCallableMock.attach_mock(mock: NonCallableMock); call it with the wrong type.

typeshed contract: mock is NonCallableMock. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from unittest.mock import NonCallableMock
obj = object.__new__(NonCallableMock)
try:
    obj.attach_mock(_W(), "")  # mock: NonCallableMock <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
"###);
    assert!(out == "STRICT_TYPE_REJECTED" || out.starts_with("RUNTIME_REJECTED"),
        "type wall did not hold: {out}");
}
