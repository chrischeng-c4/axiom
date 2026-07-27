use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_003.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_003() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_003"
# subject = "cpython.test_patma.TestPatma.test_patma_003"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_003
"""Auto-ported test: TestPatma::test_patma_003 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = False
match 0:
    case 0 | 1 | 2 | 3:
        x = True

assert x is True
print("TestPatma::test_patma_003: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_003: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_004.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_004() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_004"
# subject = "cpython.test_patma.TestPatma.test_patma_004"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_004
"""Auto-ported test: TestPatma::test_patma_004 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = False
match 1:
    case 0 | 1 | 2 | 3:
        x = True

assert x is True
print("TestPatma::test_patma_004: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_004: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_005.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_005() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_005"
# subject = "cpython.test_patma.TestPatma.test_patma_005"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_005
"""Auto-ported test: TestPatma::test_patma_005 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = False
match 2:
    case 0 | 1 | 2 | 3:
        x = True

assert x is True
print("TestPatma::test_patma_005: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_005: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_006.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_006() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_006"
# subject = "cpython.test_patma.TestPatma.test_patma_006"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_006
"""Auto-ported test: TestPatma::test_patma_006 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = False
match 3:
    case 0 | 1 | 2 | 3:
        x = True

assert x is True
print("TestPatma::test_patma_006: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_006: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_007.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_007() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_007"
# subject = "cpython.test_patma.TestPatma.test_patma_007"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_007
"""Auto-ported test: TestPatma::test_patma_007 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = False
match 4:
    case 0 | 1 | 2 | 3:
        x = True

assert x is False
print("TestPatma::test_patma_007: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_007: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_008.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_008() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_008"
# subject = "cpython.test_patma.TestPatma.test_patma_008"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_008
"""Auto-ported test: TestPatma::test_patma_008 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 0

class A:
    y = 1
match x:
    case A.y as z:
        pass

assert x == 0

assert A.y == 1
print("TestPatma::test_patma_008: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_008: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_024.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_024() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_024"
# subject = "cpython.test_patma.TestPatma.test_patma_024"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_024
"""Auto-ported test: TestPatma::test_patma_024 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = {}
y = None
match x:
    case {0: 0}:
        y = 0

assert x == {}

assert y is None
print("TestPatma::test_patma_024: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_024: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_028.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_028() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_028"
# subject = "cpython.test_patma.TestPatma.test_patma_028"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_028
"""Auto-ported test: TestPatma::test_patma_028 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = {0: 3}
y = None
match x:
    case {0: 0 | 1 | 2 as z}:
        y = 0

assert x == {0: 3}

assert y is None
print("TestPatma::test_patma_028: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_028: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_029.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_029() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_029"
# subject = "cpython.test_patma.TestPatma.test_patma_029"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_029
"""Auto-ported test: TestPatma::test_patma_029 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = {}
y = None
match x:
    case {0: [1, 2, {}]}:
        y = 0
    case {0: [1, 2, {}], 1: [[]]}:
        y = 1
    case []:
        y = 2

assert x == {}

assert y is None
print("TestPatma::test_patma_029: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_029: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_039.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_039() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_039"
# subject = "cpython.test_patma.TestPatma.test_patma_039"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_039
"""Auto-ported test: TestPatma::test_patma_039 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 3
y = None
match x:
    case 0 | 1 | 2:
        y = 0

assert x == 3

assert y is None
print("TestPatma::test_patma_039: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_039: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_043.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_043() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_043"
# subject = "cpython.test_patma.TestPatma.test_patma_043"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_043
"""Auto-ported test: TestPatma::test_patma_043 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 3
y = None
match x:
    case (0 as z) | (1 as z) | (2 as z) if z == x % 2:
        y = 0

assert x == 3

assert y is None
print("TestPatma::test_patma_043: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_043: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_050.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_050() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_050"
# subject = "cpython.test_patma.TestPatma.test_patma_050"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_050
"""Auto-ported test: TestPatma::test_patma_050 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = [0, 0]
y = None
match x:
    case [0, 1] | [1, 0]:
        y = 0

assert x == [0, 0]

assert y is None
print("TestPatma::test_patma_050: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_050: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_053.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_053() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_053"
# subject = "cpython.test_patma.TestPatma.test_patma_053"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_053
"""Auto-ported test: TestPatma::test_patma_053 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = {0}
y = None
match x:
    case [0]:
        y = 0

assert x == {0}

assert y is None
print("TestPatma::test_patma_053: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_053: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_054.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_054() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_054"
# subject = "cpython.test_patma.TestPatma.test_patma_054"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_054
"""Auto-ported test: TestPatma::test_patma_054 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = set()
y = None
match x:
    case []:
        y = 0

assert x == set()

assert y is None
print("TestPatma::test_patma_054: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_054: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_056.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_056() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_056"
# subject = "cpython.test_patma.TestPatma.test_patma_056"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_056
"""Auto-ported test: TestPatma::test_patma_056 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = {}
y = None
match x:
    case []:
        y = 0

assert x == {}

assert y is None
print("TestPatma::test_patma_056: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_056: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_057.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_057() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_057"
# subject = "cpython.test_patma.TestPatma.test_patma_057"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_057
"""Auto-ported test: TestPatma::test_patma_057 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = {0: False, 1: True}
y = None
match x:
    case [0, 1]:
        y = 0

assert x == {0: False, 1: True}

assert y is None
print("TestPatma::test_patma_057: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_057: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_060.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_060() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_060"
# subject = "cpython.test_patma.TestPatma.test_patma_060"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_060
"""Auto-ported test: TestPatma::test_patma_060 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 0
y = None
match x:
    case 1:
        y = 0

assert x == 0

assert y is None
print("TestPatma::test_patma_060: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_060: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_061.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_061() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_061"
# subject = "cpython.test_patma.TestPatma.test_patma_061"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_061
"""Auto-ported test: TestPatma::test_patma_061 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 0
y = None
match x:
    case None:
        y = 0

assert x == 0

assert y is None
print("TestPatma::test_patma_061: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_061: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_063.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_063() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_063"
# subject = "cpython.test_patma.TestPatma.test_patma_063"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_063
"""Auto-ported test: TestPatma::test_patma_063 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 0
y = None
match x:
    case 1:
        y = 0
    case 1:
        y = 1

assert x == 0

assert y is None
print("TestPatma::test_patma_063: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_063: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_072.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_072() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_072"
# subject = "cpython.test_patma.TestPatma.test_patma_072"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_072
"""Auto-ported test: TestPatma::test_patma_072 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 0
match x:
    case 0 if True:
        y = 0
    case 0 if True:
        y = 1
y = 2

assert x == 0

assert y == 2
print("TestPatma::test_patma_072: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_072: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_074.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_074() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_074"
# subject = "cpython.test_patma.TestPatma.test_patma_074"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_074
"""Auto-ported test: TestPatma::test_patma_074 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 0
y = None
match x:
    case 0 if not (x := 1):
        y = 0
    case 1:
        y = 1

assert x == 1

assert y is None
print("TestPatma::test_patma_074: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_074: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_077.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_077() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_077"
# subject = "cpython.test_patma.TestPatma.test_patma_077"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_077
"""Auto-ported test: TestPatma::test_patma_077 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = bytearray(b'x')
y = None
match x:
    case [120]:
        y = 0
    case 120:
        y = 1

assert x == b'x'

assert y is None
print("TestPatma::test_patma_077: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_077: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_084.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_084() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_084"
# subject = "cpython.test_patma.TestPatma.test_patma_084"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_084
"""Auto-ported test: TestPatma::test_patma_084 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 0
y = None
match x:
    case 1 as z:
        y = 0

assert x == 0

assert y is None
print("TestPatma::test_patma_084: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_084: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_090.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_090() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_090"
# subject = "cpython.test_patma.TestPatma.test_patma_090"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_090
"""Auto-ported test: TestPatma::test_patma_090 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 3
y = None
match x:
    case (0 | 1) | 2:
        y = 0

assert x == 3

assert y is None
print("TestPatma::test_patma_090: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_090: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_094.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_094() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_094"
# subject = "cpython.test_patma.TestPatma.test_patma_094"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_094
"""Auto-ported test: TestPatma::test_patma_094 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 3
y = None
match x:
    case 0 | (1 | 2):
        y = 0

assert x == 3

assert y is None
print("TestPatma::test_patma_094: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_094: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_155.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_155() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_155"
# subject = "cpython.test_patma.TestPatma.test_patma_155"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_155
"""Auto-ported test: TestPatma::test_patma_155 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 0
y = None
match x:
    case 1e309:
        y = 0

assert x == 0

assert y is None
print("TestPatma::test_patma_155: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_155: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_163.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_163() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_163"
# subject = "cpython.test_patma.TestPatma.test_patma_163"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_163
"""Auto-ported test: TestPatma::test_patma_163 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
x = 0
y = None
match x:
    case 1:
        y = 0
    case 1 if not x:
        y = 1

assert x == 0

assert y is None
print("TestPatma::test_patma_163: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_163: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_174.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_174() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_174"
# subject = "cpython.test_patma.TestPatma.test_patma_174"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_174
"""Auto-ported test: TestPatma::test_patma_174 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def http_error(status):
    match status:
        case 400:
            return 'Bad request'
        case 401:
            return 'Unauthorized'
        case 403:
            return 'Forbidden'
        case 404:
            return 'Not found'
        case 418:
            return "I'm a teapot"
        case _:
            return 'Something else'

assert http_error(400) == 'Bad request'

assert http_error(401) == 'Unauthorized'

assert http_error(403) == 'Forbidden'

assert http_error(404) == 'Not found'

assert http_error(418) == "I'm a teapot"

assert http_error(123) == 'Something else'

assert http_error('400') == 'Something else'

assert http_error(401 | 403 | 404) == 'Something else'
print("TestPatma::test_patma_174: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_174: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_176.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_176() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_176"
# subject = "cpython.test_patma.TestPatma.test_patma_176"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_176
"""Auto-ported test: TestPatma::test_patma_176 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def whereis(point):
    match point:
        case [0, 0]:
            return 'Origin'
        case [0, y]:
            return f'Y={y}'
        case [x, 0]:
            return f'X={x}'
        case [x, y]:
            return f'X={x}, Y={y}'
        case _:
            return 'Not a point'

assert whereis((0, 0)) == 'Origin'

assert whereis((0, -1.0)) == 'Y=-1.0'

assert whereis(('X', 0)) == 'X=X'

assert whereis((None, 1j)) == 'X=None, Y=1j'

assert whereis(42) == 'Not a point'
print("TestPatma::test_patma_176: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_176: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_208.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_208() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_208"
# subject = "cpython.test_patma.TestPatma.test_patma_208"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_208
"""Auto-ported test: TestPatma::test_patma_208 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def f(w):
    match w:
        case x:
            out = locals()
            del out['w']
            return out

assert f(42) == {'x': 42}

assert f((1, 2)) == {'x': (1, 2)}

assert f(None) == {'x': None}
print("TestPatma::test_patma_208: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_208: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_209.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_209() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_209"
# subject = "cpython.test_patma.TestPatma.test_patma_209"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_209
"""Auto-ported test: TestPatma::test_patma_209 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def f(w):
    match w:
        case _:
            out = locals()
            del out['w']
            return out

assert f(42) == {}

assert f(None) == {}

assert f((1, 2)) == {}
print("TestPatma::test_patma_209: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_209: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_214.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_214() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_214"
# subject = "cpython.test_patma.TestPatma.test_patma_214"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_214
"""Auto-ported test: TestPatma::test_patma_214 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def f():
    match 42:
        case 42:
            return locals()

assert set(f()) == set()
print("TestPatma::test_patma_214: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_214: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_215.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_215() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_215"
# subject = "cpython.test_patma.TestPatma.test_patma_215"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_215
"""Auto-ported test: TestPatma::test_patma_215 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def f():
    match 1:
        case 1 | 2 | 3:
            return locals()

assert set(f()) == set()
print("TestPatma::test_patma_215: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_215: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_222.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_222() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_222"
# subject = "cpython.test_patma.TestPatma.test_patma_222"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_222
"""Auto-ported test: TestPatma::test_patma_222 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def f(x):
    match x:
        case _:
            return 0

assert f(0) == 0

assert f(1) == 0

assert f(2) == 0

assert f(3) == 0
print("TestPatma::test_patma_222: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_222: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_224.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_224() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_224"
# subject = "cpython.test_patma.TestPatma.test_patma_224"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_224
"""Auto-ported test: TestPatma::test_patma_224 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def f(x):
    match x:
        case 0:
            return 0
        case _:
            return 1

assert f(0) == 0

assert f(1) == 1

assert f(2) == 1

assert f(3) == 1
print("TestPatma::test_patma_224: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_224: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_226.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_226() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_226"
# subject = "cpython.test_patma.TestPatma.test_patma_226"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_226
"""Auto-ported test: TestPatma::test_patma_226 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def f(x):
    match x:
        case 0:
            return 0
        case 1:
            return 1
        case _:
            return 2

assert f(0) == 0

assert f(1) == 1

assert f(2) == 2

assert f(3) == 2
print("TestPatma::test_patma_226: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_226: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_232.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_232() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_232"
# subject = "cpython.test_patma.TestPatma.test_patma_232"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_232
"""Auto-ported test: TestPatma::test_patma_232 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
class Eq:

    def __eq__(self, other):
        return True
x = eq = Eq()
y = None
match x:
    case None:
        y = 0

assert x is eq

assert y == None
print("TestPatma::test_patma_232: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_232: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_patma__test_patma_249.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_patma__test_patma_249() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_249"
# subject = "cpython.test_patma.TestPatma.test_patma_249"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_249
"""Auto-ported test: TestPatma::test_patma_249 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
class C:
    __attr = 'eggs'
    _Outer__attr = 'bacon'

class Outer:

    def f(self, x):
        match x:
            case C(__attr=y):
                return y
c = C()
setattr(c, '__attr', 'spam')

assert Outer().f(c) == 'spam'
print("TestPatma::test_patma_249: ok")
"###);
    assert_output(&out, r###"TestPatma::test_patma_249: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_source_locations__test_jump_threading.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_source_locations__test_jump_threading() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_source_locations__test_jump_threading"
# subject = "cpython.test_patma.TestSourceLocations.test_jump_threading"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSourceLocations::test_jump_threading
"""Auto-ported test: TestSourceLocations::test_jump_threading (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def f():
    x = 0
    v = 1
    match v:
        case 1:
            if x < 0:
                x = 1
        case 2:
            if x < 0:
                x = 1
    x += 1
for inst in dis.get_instructions(f):
    if inst.opcode in dis.hasjrel or inst.opcode in dis.hasjabs:

        assert inst.positions.lineno is not None
print("TestSourceLocations::test_jump_threading: ok")
"###);
    assert_output(&out, r###"TestSourceLocations::test_jump_threading: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_alternative_patterns_bind_different_names_0.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_alternative_patterns_bind_different_names_0() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_alternative_patterns_bind_different_names_0"
# subject = "cpython.test_patma.TestSyntaxErrors.test_alternative_patterns_bind_different_names_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_alternative_patterns_bind_different_names_0
"""Auto-ported test: TestSyntaxErrors::test_alternative_patterns_bind_different_names_0 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case "a" | a:\n                pass\n        ')
print("TestSyntaxErrors::test_alternative_patterns_bind_different_names_0: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_alternative_patterns_bind_different_names_0: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_alternative_patterns_bind_different_names_1.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_alternative_patterns_bind_different_names_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_alternative_patterns_bind_different_names_1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_alternative_patterns_bind_different_names_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_alternative_patterns_bind_different_names_1
"""Auto-ported test: TestSyntaxErrors::test_alternative_patterns_bind_different_names_1 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case [a, [b] | [c] | [d]]:\n                pass\n        ')
print("TestSyntaxErrors::test_alternative_patterns_bind_different_names_1: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_alternative_patterns_bind_different_names_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_attribute_name_repeated_in_class_pattern.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_attribute_name_repeated_in_class_pattern() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_attribute_name_repeated_in_class_pattern"
# subject = "cpython.test_patma.TestSyntaxErrors.test_attribute_name_repeated_in_class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_attribute_name_repeated_in_class_pattern
"""Auto-ported test: TestSyntaxErrors::test_attribute_name_repeated_in_class_pattern (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case Class(a=_, a=_):\n                pass\n        ')
print("TestSyntaxErrors::test_attribute_name_repeated_in_class_pattern: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_attribute_name_repeated_in_class_pattern: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_imaginary_number_required_in_complex_literal_0.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_imaginary_number_required_in_complex_literal_0() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_imaginary_number_required_in_complex_literal_0"
# subject = "cpython.test_patma.TestSyntaxErrors.test_imaginary_number_required_in_complex_literal_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_imaginary_number_required_in_complex_literal_0
"""Auto-ported test: TestSyntaxErrors::test_imaginary_number_required_in_complex_literal_0 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case 0+0:\n                pass\n        ')
print("TestSyntaxErrors::test_imaginary_number_required_in_complex_literal_0: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_imaginary_number_required_in_complex_literal_0: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_imaginary_number_required_in_complex_literal_1.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_imaginary_number_required_in_complex_literal_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_imaginary_number_required_in_complex_literal_1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_imaginary_number_required_in_complex_literal_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_imaginary_number_required_in_complex_literal_1
"""Auto-ported test: TestSyntaxErrors::test_imaginary_number_required_in_complex_literal_1 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {0+0: _}:\n                pass\n        ')
print("TestSyntaxErrors::test_imaginary_number_required_in_complex_literal_1: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_imaginary_number_required_in_complex_literal_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_invalid_syntax_0.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_invalid_syntax_0() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_invalid_syntax_0"
# subject = "cpython.test_patma.TestSyntaxErrors.test_invalid_syntax_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_invalid_syntax_0
"""Auto-ported test: TestSyntaxErrors::test_invalid_syntax_0 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {**rest, "key": value}:\n                pass\n        ')
print("TestSyntaxErrors::test_invalid_syntax_0: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_invalid_syntax_0: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_invalid_syntax_1.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_invalid_syntax_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_invalid_syntax_1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_invalid_syntax_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_invalid_syntax_1
"""Auto-ported test: TestSyntaxErrors::test_invalid_syntax_1 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {"first": first, **rest, "last": last}:\n                pass\n        ')
print("TestSyntaxErrors::test_invalid_syntax_1: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_invalid_syntax_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_invalid_syntax_2.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_invalid_syntax_2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_invalid_syntax_2"
# subject = "cpython.test_patma.TestSyntaxErrors.test_invalid_syntax_2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_invalid_syntax_2
"""Auto-ported test: TestSyntaxErrors::test_invalid_syntax_2 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {**_}:\n                pass\n        ')
print("TestSyntaxErrors::test_invalid_syntax_2: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_invalid_syntax_2: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_invalid_syntax_3.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_invalid_syntax_3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_invalid_syntax_3"
# subject = "cpython.test_patma.TestSyntaxErrors.test_invalid_syntax_3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_invalid_syntax_3
"""Auto-ported test: TestSyntaxErrors::test_invalid_syntax_3 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case 42 as _:\n                pass\n        ')
print("TestSyntaxErrors::test_invalid_syntax_3: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_invalid_syntax_3: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_mapping_pattern_duplicate_key.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_mapping_pattern_duplicate_key() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_mapping_pattern_duplicate_key"
# subject = "cpython.test_patma.TestSyntaxErrors.test_mapping_pattern_duplicate_key"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_mapping_pattern_duplicate_key
"""Auto-ported test: TestSyntaxErrors::test_mapping_pattern_duplicate_key (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {"a": _, "a": _}:\n                pass\n        ')
print("TestSyntaxErrors::test_mapping_pattern_duplicate_key: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_mapping_pattern_duplicate_key: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case0.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case0() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case0"
# subject = "cpython.test_patma.TestSyntaxErrors.test_mapping_pattern_duplicate_key_edge_case0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case0
"""Auto-ported test: TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case0 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {0: _, False: _}:\n                pass\n        ')
print("TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case0: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case0: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case1.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_mapping_pattern_duplicate_key_edge_case1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case1
"""Auto-ported test: TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case1 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {0: _, 0.0: _}:\n                pass\n        ')
print("TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case1: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case1: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case2.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case2"
# subject = "cpython.test_patma.TestSyntaxErrors.test_mapping_pattern_duplicate_key_edge_case2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case2
"""Auto-ported test: TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case2 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {0: _, -0: _}:\n                pass\n        ')
print("TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case2: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case2: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case3.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_mapping_pattern_duplicate_key_edge_case3"
# subject = "cpython.test_patma.TestSyntaxErrors.test_mapping_pattern_duplicate_key_edge_case3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case3
"""Auto-ported test: TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case3 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {0: _, 0j: _}:\n                pass\n        ')
print("TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case3: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_mapping_pattern_duplicate_key_edge_case3: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_mapping_pattern_keys_may_only_match_literals_and_attribute_lookups.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_mapping_pattern_keys_may_only_match_literals_and_attribute_lookups() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_mapping_pattern_keys_may_only_match_literals_and_attribute_lookups"
# subject = "cpython.test_patma.TestSyntaxErrors.test_mapping_pattern_keys_may_only_match_literals_and_attribute_lookups"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_mapping_pattern_keys_may_only_match_literals_and_attribute_lookups
"""Auto-ported test: TestSyntaxErrors::test_mapping_pattern_keys_may_only_match_literals_and_attribute_lookups (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {f"": _}:\n                pass\n        ')
print("TestSyntaxErrors::test_mapping_pattern_keys_may_only_match_literals_and_attribute_lookups: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_mapping_pattern_keys_may_only_match_literals_and_attribute_lookups: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_multiple_assignments_to_name_in_pattern_0.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_multiple_assignments_to_name_in_pattern_0() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_multiple_assignments_to_name_in_pattern_0"
# subject = "cpython.test_patma.TestSyntaxErrors.test_multiple_assignments_to_name_in_pattern_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_0
"""Auto-ported test: TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_0 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case a, a:\n                pass\n        ')
print("TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_0: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_0: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_multiple_assignments_to_name_in_pattern_1.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_multiple_assignments_to_name_in_pattern_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_multiple_assignments_to_name_in_pattern_1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_multiple_assignments_to_name_in_pattern_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_1
"""Auto-ported test: TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_1 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {"k": a, "l": a}:\n                pass\n        ')
print("TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_1: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_multiple_assignments_to_name_in_pattern_2.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_multiple_assignments_to_name_in_pattern_2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_multiple_assignments_to_name_in_pattern_2"
# subject = "cpython.test_patma.TestSyntaxErrors.test_multiple_assignments_to_name_in_pattern_2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_2
"""Auto-ported test: TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_2 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case MyClass(x, x):\n                pass\n        ')
print("TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_2: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_2: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_multiple_assignments_to_name_in_pattern_3.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_multiple_assignments_to_name_in_pattern_3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_multiple_assignments_to_name_in_pattern_3"
# subject = "cpython.test_patma.TestSyntaxErrors.test_multiple_assignments_to_name_in_pattern_3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_3
"""Auto-ported test: TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_3 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case MyClass(x=x, y=x):\n                pass\n        ')
print("TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_3: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_3: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_multiple_assignments_to_name_in_pattern_4.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_multiple_assignments_to_name_in_pattern_4() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_multiple_assignments_to_name_in_pattern_4"
# subject = "cpython.test_patma.TestSyntaxErrors.test_multiple_assignments_to_name_in_pattern_4"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_4
"""Auto-ported test: TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_4 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case MyClass(x, y=x):\n                pass\n        ')
print("TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_4: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_4: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_multiple_assignments_to_name_in_pattern_5.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_multiple_assignments_to_name_in_pattern_5() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_multiple_assignments_to_name_in_pattern_5"
# subject = "cpython.test_patma.TestSyntaxErrors.test_multiple_assignments_to_name_in_pattern_5"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_5
"""Auto-ported test: TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_5 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case a as a:\n                pass\n        ')
print("TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_5: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_multiple_assignments_to_name_in_pattern_5: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_multiple_starred_names_in_sequence_pattern_0.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_multiple_starred_names_in_sequence_pattern_0() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_multiple_starred_names_in_sequence_pattern_0"
# subject = "cpython.test_patma.TestSyntaxErrors.test_multiple_starred_names_in_sequence_pattern_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_multiple_starred_names_in_sequence_pattern_0
"""Auto-ported test: TestSyntaxErrors::test_multiple_starred_names_in_sequence_pattern_0 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case *a, b, *c, d, *e:\n                pass\n        ')
print("TestSyntaxErrors::test_multiple_starred_names_in_sequence_pattern_0: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_multiple_starred_names_in_sequence_pattern_0: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_multiple_starred_names_in_sequence_pattern_1.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_multiple_starred_names_in_sequence_pattern_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_multiple_starred_names_in_sequence_pattern_1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_multiple_starred_names_in_sequence_pattern_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_multiple_starred_names_in_sequence_pattern_1
"""Auto-ported test: TestSyntaxErrors::test_multiple_starred_names_in_sequence_pattern_1 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case a, *b, c, *d, e:\n                pass\n        ')
print("TestSyntaxErrors::test_multiple_starred_names_in_sequence_pattern_1: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_multiple_starred_names_in_sequence_pattern_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_0.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_0() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_0"
# subject = "cpython.test_patma.TestSyntaxErrors.test_name_capture_makes_remaining_patterns_unreachable_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_0
"""Auto-ported test: TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_0 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case a | "a":\n                pass\n        ')
print("TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_0: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_0: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_1.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_name_capture_makes_remaining_patterns_unreachable_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_1
"""Auto-ported test: TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_1 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match 42:\n            case x:\n                pass\n            case y:\n                pass\n        ')
print("TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_1: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_2.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_2"
# subject = "cpython.test_patma.TestSyntaxErrors.test_name_capture_makes_remaining_patterns_unreachable_2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_2
"""Auto-ported test: TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_2 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case x | [_ as x] if x:\n                pass\n        ')
print("TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_2: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_2: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_3.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_3"
# subject = "cpython.test_patma.TestSyntaxErrors.test_name_capture_makes_remaining_patterns_unreachable_3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_3
"""Auto-ported test: TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_3 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case x:\n                pass\n            case [x] if x:\n                pass\n        ')
print("TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_3: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_3: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_4.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_4() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_name_capture_makes_remaining_patterns_unreachable_4"
# subject = "cpython.test_patma.TestSyntaxErrors.test_name_capture_makes_remaining_patterns_unreachable_4"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_4
"""Auto-ported test: TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_4 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case x:\n                pass\n            case _:\n                pass\n        ')
print("TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_4: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_name_capture_makes_remaining_patterns_unreachable_4: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_patterns_may_only_match_literals_and_attribute_lookups_0.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_patterns_may_only_match_literals_and_attribute_lookups_0() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_patterns_may_only_match_literals_and_attribute_lookups_0"
# subject = "cpython.test_patma.TestSyntaxErrors.test_patterns_may_only_match_literals_and_attribute_lookups_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_patterns_may_only_match_literals_and_attribute_lookups_0
"""Auto-ported test: TestSyntaxErrors::test_patterns_may_only_match_literals_and_attribute_lookups_0 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case f"":\n                pass\n        ')
print("TestSyntaxErrors::test_patterns_may_only_match_literals_and_attribute_lookups_0: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_patterns_may_only_match_literals_and_attribute_lookups_0: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_patterns_may_only_match_literals_and_attribute_lookups_1.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_patterns_may_only_match_literals_and_attribute_lookups_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_patterns_may_only_match_literals_and_attribute_lookups_1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_patterns_may_only_match_literals_and_attribute_lookups_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_patterns_may_only_match_literals_and_attribute_lookups_1
"""Auto-ported test: TestSyntaxErrors::test_patterns_may_only_match_literals_and_attribute_lookups_1 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case f"{x}":\n                pass\n        ')
print("TestSyntaxErrors::test_patterns_may_only_match_literals_and_attribute_lookups_1: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_patterns_may_only_match_literals_and_attribute_lookups_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_real_number_required_in_complex_literal_0.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_real_number_required_in_complex_literal_0() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_real_number_required_in_complex_literal_0"
# subject = "cpython.test_patma.TestSyntaxErrors.test_real_number_required_in_complex_literal_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_real_number_required_in_complex_literal_0
"""Auto-ported test: TestSyntaxErrors::test_real_number_required_in_complex_literal_0 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case 0j+0:\n                pass\n        ')
print("TestSyntaxErrors::test_real_number_required_in_complex_literal_0: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_real_number_required_in_complex_literal_0: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_real_number_required_in_complex_literal_1.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_real_number_required_in_complex_literal_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_real_number_required_in_complex_literal_1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_real_number_required_in_complex_literal_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_real_number_required_in_complex_literal_1
"""Auto-ported test: TestSyntaxErrors::test_real_number_required_in_complex_literal_1 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case 0j+0j:\n                pass\n        ')
print("TestSyntaxErrors::test_real_number_required_in_complex_literal_1: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_real_number_required_in_complex_literal_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_real_number_required_in_complex_literal_2.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_real_number_required_in_complex_literal_2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_real_number_required_in_complex_literal_2"
# subject = "cpython.test_patma.TestSyntaxErrors.test_real_number_required_in_complex_literal_2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_real_number_required_in_complex_literal_2
"""Auto-ported test: TestSyntaxErrors::test_real_number_required_in_complex_literal_2 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {0j+0: _}:\n                pass\n        ')
print("TestSyntaxErrors::test_real_number_required_in_complex_literal_2: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_real_number_required_in_complex_literal_2: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_real_number_required_in_complex_literal_3.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_real_number_required_in_complex_literal_3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_real_number_required_in_complex_literal_3"
# subject = "cpython.test_patma.TestSyntaxErrors.test_real_number_required_in_complex_literal_3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_real_number_required_in_complex_literal_3
"""Auto-ported test: TestSyntaxErrors::test_real_number_required_in_complex_literal_3 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case {0j+0j: _}:\n                pass\n        ')
print("TestSyntaxErrors::test_real_number_required_in_complex_literal_3: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_real_number_required_in_complex_literal_3: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_0.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_0() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_0"
# subject = "cpython.test_patma.TestSyntaxErrors.test_wildcard_makes_remaining_patterns_unreachable_0"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_0
"""Auto-ported test: TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_0 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case _ | _:\n                pass\n        ')
print("TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_0: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_0: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_1.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_1() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_1"
# subject = "cpython.test_patma.TestSyntaxErrors.test_wildcard_makes_remaining_patterns_unreachable_1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_1
"""Auto-ported test: TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_1 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case (_ as x) | [x]:\n                pass\n        ')
print("TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_1: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_1: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_2.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_2() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_2"
# subject = "cpython.test_patma.TestSyntaxErrors.test_wildcard_makes_remaining_patterns_unreachable_2"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_2
"""Auto-ported test: TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_2 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case _ | _ if condition():\n                pass\n        ')
print("TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_2: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_2: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_3.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_3"
# subject = "cpython.test_patma.TestSyntaxErrors.test_wildcard_makes_remaining_patterns_unreachable_3"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_3
"""Auto-ported test: TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_3 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case _:\n                pass\n            case None:\n                pass\n        ')
print("TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_3: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_3: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_4.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_4() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_4"
# subject = "cpython.test_patma.TestSyntaxErrors.test_wildcard_makes_remaining_patterns_unreachable_4"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_4
"""Auto-ported test: TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_4 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case (None | _) | _:\n                pass\n        ')
print("TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_4: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_4: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/pattern_matching/test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_5.py`.
#[test]
fn test_gen_behavior_core_pattern_matching_test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_5() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_syntax_errors__test_wildcard_makes_remaining_patterns_unreachable_5"
# subject = "cpython.test_patma.TestSyntaxErrors.test_wildcard_makes_remaining_patterns_unreachable_5"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_5
"""Auto-ported test: TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_5 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def assert_syntax_error(code: str):
    try:
        compile(inspect.cleandoc(code), '<test>', 'exec')
        raise AssertionError('expected SyntaxError')
    except SyntaxError:
        pass
assert_syntax_error('\n        match ...:\n            case _ | (True | False):\n                pass\n        ')
print("TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_5: ok")
"###);
    assert_output(&out, r###"TestSyntaxErrors::test_wildcard_makes_remaining_patterns_unreachable_5: ok
"###);
}
