use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/builtin-libs/enumerate/enumerate_start_test_case__test_enumerate_result_gc.py`.
#[test]
fn test_gen_behavior_builtin_libs_enumerate_enumerate_start_test_case__test_enumerate_result_gc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "enumerate"
# dimension = "behavior"
# case = "enumerate_start_test_case__test_enumerate_result_gc"
# subject = "cpython.test.test_enumerate.EnumerateStartTestCase.test_enumerate_result_gc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enumerate.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_enumerate.py::EnumerateStartTestCase::test_enumerate_result_gc
"""Auto-ported test: EnumerateStartTestCase::test_enumerate_result_gc (CPython 3.12 oracle)."""


import unittest
import operator
import sys
import pickle
import gc
from test import support


class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class PickleTest:

    def check_pickle(self, itorg, seq):
        for proto in range(pickle.HIGHEST_PROTOCOL + 1):
            d = pickle.dumps(itorg, proto)
            it = pickle.loads(d)
            self.assertEqual(type(itorg), type(it))
            self.assertEqual(list(it), seq)
            it = pickle.loads(d)
            try:
                next(it)
            except StopIteration:
                self.assertFalse(seq[1:])
                continue
            d = pickle.dumps(it, proto)
            it = pickle.loads(d)
            self.assertEqual(list(it), seq[1:])

class MyEnum(enumerate):
    pass


# --- test body ---
enum = enumerate

def check_pickle(itorg, seq):
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        d = pickle.dumps(itorg, proto)
        it = pickle.loads(d)

        assert type(itorg) == type(it)

        assert list(it) == seq
        it = pickle.loads(d)
        try:
            next(it)
        except StopIteration:

            assert not seq[1:]
            continue
        d = pickle.dumps(it, proto)
        it = pickle.loads(d)

        assert list(it) == seq[1:]
it = enum([[]])
gc.collect()

assert gc.is_tracked(next(it))
print("EnumerateStartTestCase::test_enumerate_result_gc: ok")
"###);
    assert_output(&out, r###"EnumerateStartTestCase::test_enumerate_result_gc: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/enumerate/enumerate_test_case__test_enumerate_result_gc.py`.
#[test]
fn test_gen_behavior_builtin_libs_enumerate_enumerate_test_case__test_enumerate_result_gc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "enumerate"
# dimension = "behavior"
# case = "enumerate_test_case__test_enumerate_result_gc"
# subject = "cpython.test.test_enumerate.EnumerateTestCase.test_enumerate_result_gc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enumerate.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_enumerate.py::EnumerateTestCase::test_enumerate_result_gc
"""Auto-ported test: EnumerateTestCase::test_enumerate_result_gc (CPython 3.12 oracle)."""


import unittest
import operator
import sys
import pickle
import gc
from test import support


class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class PickleTest:

    def check_pickle(self, itorg, seq):
        for proto in range(pickle.HIGHEST_PROTOCOL + 1):
            d = pickle.dumps(itorg, proto)
            it = pickle.loads(d)
            self.assertEqual(type(itorg), type(it))
            self.assertEqual(list(it), seq)
            it = pickle.loads(d)
            try:
                next(it)
            except StopIteration:
                self.assertFalse(seq[1:])
                continue
            d = pickle.dumps(it, proto)
            it = pickle.loads(d)
            self.assertEqual(list(it), seq[1:])

class MyEnum(enumerate):
    pass


# --- test body ---
enum = enumerate

def check_pickle(itorg, seq):
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        d = pickle.dumps(itorg, proto)
        it = pickle.loads(d)

        assert type(itorg) == type(it)

        assert list(it) == seq
        it = pickle.loads(d)
        try:
            next(it)
        except StopIteration:

            assert not seq[1:]
            continue
        d = pickle.dumps(it, proto)
        it = pickle.loads(d)

        assert list(it) == seq[1:]
it = enum([[]])
gc.collect()

assert gc.is_tracked(next(it))
print("EnumerateTestCase::test_enumerate_result_gc: ok")
"###);
    assert_output(&out, r###"EnumerateTestCase::test_enumerate_result_gc: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/enumerate/test_big__test_enumerate_result_gc.py`.
#[test]
fn test_gen_behavior_builtin_libs_enumerate_test_big__test_enumerate_result_gc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "enumerate"
# dimension = "behavior"
# case = "test_big__test_enumerate_result_gc"
# subject = "cpython.test.test_enumerate.TestBig.test_enumerate_result_gc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enumerate.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_enumerate.py::TestBig::test_enumerate_result_gc
"""Auto-ported test: TestBig::test_enumerate_result_gc (CPython 3.12 oracle)."""


import unittest
import operator
import sys
import pickle
import gc
from test import support


class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class PickleTest:

    def check_pickle(self, itorg, seq):
        for proto in range(pickle.HIGHEST_PROTOCOL + 1):
            d = pickle.dumps(itorg, proto)
            it = pickle.loads(d)
            self.assertEqual(type(itorg), type(it))
            self.assertEqual(list(it), seq)
            it = pickle.loads(d)
            try:
                next(it)
            except StopIteration:
                self.assertFalse(seq[1:])
                continue
            d = pickle.dumps(it, proto)
            it = pickle.loads(d)
            self.assertEqual(list(it), seq[1:])

class MyEnum(enumerate):
    pass


# --- test body ---
enum = enumerate
seq = range(10, 20000, 2)
res = list(zip(range(20000), seq))

def check_pickle(itorg, seq):
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        d = pickle.dumps(itorg, proto)
        it = pickle.loads(d)

        assert type(itorg) == type(it)

        assert list(it) == seq
        it = pickle.loads(d)
        try:
            next(it)
        except StopIteration:

            assert not seq[1:]
            continue
        d = pickle.dumps(it, proto)
        it = pickle.loads(d)

        assert list(it) == seq[1:]
it = enum([[]])
gc.collect()

assert gc.is_tracked(next(it))
print("TestBig::test_enumerate_result_gc: ok")
"###);
    assert_output(&out, r###"TestBig::test_enumerate_result_gc: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/enumerate/test_big__test_noniterable.py`.
#[test]
fn test_gen_behavior_builtin_libs_enumerate_test_big__test_noniterable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "enumerate"
# dimension = "behavior"
# case = "test_big__test_noniterable"
# subject = "cpython.test.test_enumerate.TestBig.test_noniterable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enumerate.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_enumerate.py::TestBig::test_noniterable
"""Auto-ported test: TestBig::test_noniterable (CPython 3.12 oracle)."""


import unittest
import operator
import sys
import pickle
import gc
from test import support


class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class PickleTest:

    def check_pickle(self, itorg, seq):
        for proto in range(pickle.HIGHEST_PROTOCOL + 1):
            d = pickle.dumps(itorg, proto)
            it = pickle.loads(d)
            self.assertEqual(type(itorg), type(it))
            self.assertEqual(list(it), seq)
            it = pickle.loads(d)
            try:
                next(it)
            except StopIteration:
                self.assertFalse(seq[1:])
                continue
            d = pickle.dumps(it, proto)
            it = pickle.loads(d)
            self.assertEqual(list(it), seq[1:])

class MyEnum(enumerate):
    pass


# --- test body ---
enum = enumerate
seq = range(10, 20000, 2)
res = list(zip(range(20000), seq))

def check_pickle(itorg, seq):
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        d = pickle.dumps(itorg, proto)
        it = pickle.loads(d)

        assert type(itorg) == type(it)

        assert list(it) == seq
        it = pickle.loads(d)
        try:
            next(it)
        except StopIteration:

            assert not seq[1:]
            continue
        d = pickle.dumps(it, proto)
        it = pickle.loads(d)

        assert list(it) == seq[1:]

try:
    enum(X(seq))
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("TestBig::test_noniterable: ok")
"###);
    assert_output(&out, r###"TestBig::test_noniterable: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/enumerate/test_empty__test_enumerate_result_gc.py`.
#[test]
fn test_gen_behavior_builtin_libs_enumerate_test_empty__test_enumerate_result_gc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "enumerate"
# dimension = "behavior"
# case = "test_empty__test_enumerate_result_gc"
# subject = "cpython.test.test_enumerate.TestEmpty.test_enumerate_result_gc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enumerate.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_enumerate.py::TestEmpty::test_enumerate_result_gc
"""Auto-ported test: TestEmpty::test_enumerate_result_gc (CPython 3.12 oracle)."""


import unittest
import operator
import sys
import pickle
import gc
from test import support


class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class PickleTest:

    def check_pickle(self, itorg, seq):
        for proto in range(pickle.HIGHEST_PROTOCOL + 1):
            d = pickle.dumps(itorg, proto)
            it = pickle.loads(d)
            self.assertEqual(type(itorg), type(it))
            self.assertEqual(list(it), seq)
            it = pickle.loads(d)
            try:
                next(it)
            except StopIteration:
                self.assertFalse(seq[1:])
                continue
            d = pickle.dumps(it, proto)
            it = pickle.loads(d)
            self.assertEqual(list(it), seq[1:])

class MyEnum(enumerate):
    pass


# --- test body ---
enum = enumerate

def check_pickle(itorg, seq):
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        d = pickle.dumps(itorg, proto)
        it = pickle.loads(d)

        assert type(itorg) == type(it)

        assert list(it) == seq
        it = pickle.loads(d)
        try:
            next(it)
        except StopIteration:

            assert not seq[1:]
            continue
        d = pickle.dumps(it, proto)
        it = pickle.loads(d)

        assert list(it) == seq[1:]
it = enum([[]])
gc.collect()

assert gc.is_tracked(next(it))
print("TestEmpty::test_enumerate_result_gc: ok")
"###);
    assert_output(&out, r###"TestEmpty::test_enumerate_result_gc: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/enumerate/test_reversed__test_gc.py`.
#[test]
fn test_gen_behavior_builtin_libs_enumerate_test_reversed__test_gc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "enumerate"
# dimension = "behavior"
# case = "test_reversed__test_gc"
# subject = "cpython.test.test_enumerate.TestReversed.test_gc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enumerate.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_enumerate.py::TestReversed::test_gc
"""Auto-ported test: TestReversed::test_gc (CPython 3.12 oracle)."""


import unittest
import operator
import sys
import pickle
import gc
from test import support


class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class PickleTest:

    def check_pickle(self, itorg, seq):
        for proto in range(pickle.HIGHEST_PROTOCOL + 1):
            d = pickle.dumps(itorg, proto)
            it = pickle.loads(d)
            self.assertEqual(type(itorg), type(it))
            self.assertEqual(list(it), seq)
            it = pickle.loads(d)
            try:
                next(it)
            except StopIteration:
                self.assertFalse(seq[1:])
                continue
            d = pickle.dumps(it, proto)
            it = pickle.loads(d)
            self.assertEqual(list(it), seq[1:])

class MyEnum(enumerate):
    pass


# --- test body ---
def check_pickle(itorg, seq):
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        d = pickle.dumps(itorg, proto)
        it = pickle.loads(d)

        assert type(itorg) == type(it)

        assert list(it) == seq
        it = pickle.loads(d)
        try:
            next(it)
        except StopIteration:

            assert not seq[1:]
            continue
        d = pickle.dumps(it, proto)
        it = pickle.loads(d)

        assert list(it) == seq[1:]

class Seq:

    def __len__(self):
        return 10

    def __getitem__(self, index):
        return index
s = Seq()
r = reversed(s)
s.r = r
print("TestReversed::test_gc: ok")
"###);
    assert_output(&out, r###"TestReversed::test_gc: ok
"###);
}

/// Ported from `tests/cpython/behavior/builtin-libs/enumerate/test_start__test_enumerate_result_gc.py`.
#[test]
fn test_gen_behavior_builtin_libs_enumerate_test_start__test_enumerate_result_gc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "enumerate"
# dimension = "behavior"
# case = "test_start__test_enumerate_result_gc"
# subject = "cpython.test.test_enumerate.TestStart.test_enumerate_result_gc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enumerate.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_enumerate.py::TestStart::test_enumerate_result_gc
"""Auto-ported test: TestStart::test_enumerate_result_gc (CPython 3.12 oracle)."""


import unittest
import operator
import sys
import pickle
import gc
from test import support


class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class PickleTest:

    def check_pickle(self, itorg, seq):
        for proto in range(pickle.HIGHEST_PROTOCOL + 1):
            d = pickle.dumps(itorg, proto)
            it = pickle.loads(d)
            self.assertEqual(type(itorg), type(it))
            self.assertEqual(list(it), seq)
            it = pickle.loads(d)
            try:
                next(it)
            except StopIteration:
                self.assertFalse(seq[1:])
                continue
            d = pickle.dumps(it, proto)
            it = pickle.loads(d)
            self.assertEqual(list(it), seq[1:])

class MyEnum(enumerate):
    pass


# --- test body ---
enum = enumerate

def check_pickle(itorg, seq):
    for proto in range(pickle.HIGHEST_PROTOCOL + 1):
        d = pickle.dumps(itorg, proto)
        it = pickle.loads(d)

        assert type(itorg) == type(it)

        assert list(it) == seq
        it = pickle.loads(d)
        try:
            next(it)
        except StopIteration:

            assert not seq[1:]
            continue
        d = pickle.dumps(it, proto)
        it = pickle.loads(d)

        assert list(it) == seq[1:]

def enum(iterable, start=11):
    return enumerate(iterable, start=start)
it = enum([[]])
gc.collect()

assert gc.is_tracked(next(it))
print("TestStart::test_enumerate_result_gc: ok")
"###);
    assert_output(&out, r###"TestStart::test_enumerate_result_gc: ok
"###);
}
