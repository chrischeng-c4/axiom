use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_close_with_cleared_frame.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_close_with_cleared_frame() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_close_with_cleared_frame"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_close_with_cleared_frame"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_close_with_cleared_frame
"""Auto-ported test: TestPEP380Operation::test_close_with_cleared_frame (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
def innermost():
    yield

def inner():
    outer_gen = (yield)
    yield from innermost()

def outer():
    inner_gen = (yield)
    yield from inner_gen
with disable_gc():
    inner_gen = inner()
    outer_gen = outer()
    outer_gen.send(None)
    outer_gen.send(inner_gen)
    outer_gen.send(outer_gen)
    del outer_gen
    del inner_gen
    gc_collect()
print("TestPEP380Operation::test_close_with_cleared_frame: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_close_with_cleared_frame: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_conversion_of_send_none_to_next.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_conversion_of_send_none_to_next() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_conversion_of_send_none_to_next"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_conversion_of_sendNone_to_next"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_conversion_of_sendNone_to_next
"""Auto-ported test: TestPEP380Operation::test_conversion_of_sendNone_to_next (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test conversion of send(None) to next()
        """
trace = []

def g():
    yield from range(3)
gi = g()
for x in range(3):
    y = gi.send(None)
    trace.append('Yielded: %s' % (y,))

assert trace == ['Yielded: 0', 'Yielded: 1', 'Yielded: 2']
print("TestPEP380Operation::test_conversion_of_sendNone_to_next: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_conversion_of_sendNone_to_next: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_delegating_close.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_delegating_close() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_delegating_close"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_delegating_close"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_delegating_close
"""Auto-ported test: TestPEP380Operation::test_delegating_close (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test delegating 'close'
        """
trace = []

def g1():
    try:
        trace.append('Starting g1')
        yield 'g1 ham'
        yield from g2()
        yield 'g1 eggs'
    finally:
        trace.append('Finishing g1')

def g2():
    try:
        trace.append('Starting g2')
        yield 'g2 spam'
        yield 'g2 more spam'
    finally:
        trace.append('Finishing g2')
g = g1()
for i in range(2):
    x = next(g)
    trace.append('Yielded %s' % (x,))
g.close()

assert trace == ['Starting g1', 'Yielded g1 ham', 'Starting g2', 'Yielded g2 spam', 'Finishing g2', 'Finishing g1']
print("TestPEP380Operation::test_delegating_close: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_delegating_close: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_delegating_throw.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_delegating_throw() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_delegating_throw"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_delegating_throw"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_delegating_throw
"""Auto-ported test: TestPEP380Operation::test_delegating_throw (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test delegating 'throw'
        """
trace = []

def g1():
    try:
        trace.append('Starting g1')
        yield 'g1 ham'
        yield from g2()
        yield 'g1 eggs'
    finally:
        trace.append('Finishing g1')

def g2():
    try:
        trace.append('Starting g2')
        yield 'g2 spam'
        yield 'g2 more spam'
    finally:
        trace.append('Finishing g2')
try:
    g = g1()
    for i in range(2):
        x = next(g)
        trace.append('Yielded %s' % (x,))
    e = ValueError('tomato ejected')
    g.throw(e)
except ValueError as e:

    assert e.args[0] == 'tomato ejected'
else:

    raise AssertionError('subgenerator failed to raise ValueError')

assert trace == ['Starting g1', 'Yielded g1 ham', 'Starting g2', 'Yielded g2 spam', 'Finishing g2', 'Finishing g1']
print("TestPEP380Operation::test_delegating_throw: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_delegating_throw: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_delegation_of_initial_next_to_subgenerator.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_delegation_of_initial_next_to_subgenerator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_delegation_of_initial_next_to_subgenerator"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_delegation_of_initial_next_to_subgenerator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_delegation_of_initial_next_to_subgenerator
"""Auto-ported test: TestPEP380Operation::test_delegation_of_initial_next_to_subgenerator (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test delegation of initial next() call to subgenerator
        """
trace = []

def g1():
    trace.append('Starting g1')
    yield from g2()
    trace.append('Finishing g1')

def g2():
    trace.append('Starting g2')
    yield 42
    trace.append('Finishing g2')
for x in g1():
    trace.append('Yielded %s' % (x,))

assert trace == ['Starting g1', 'Starting g2', 'Yielded 42', 'Finishing g2', 'Finishing g1']
print("TestPEP380Operation::test_delegation_of_initial_next_to_subgenerator: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_delegation_of_initial_next_to_subgenerator: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_delegation_of_next_call_to_subgenerator.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_delegation_of_next_call_to_subgenerator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_delegation_of_next_call_to_subgenerator"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_delegation_of_next_call_to_subgenerator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_delegation_of_next_call_to_subgenerator
"""Auto-ported test: TestPEP380Operation::test_delegation_of_next_call_to_subgenerator (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test delegation of next() call to subgenerator
        """
trace = []

def g1():
    trace.append('Starting g1')
    yield 'g1 ham'
    yield from g2()
    yield 'g1 eggs'
    trace.append('Finishing g1')

def g2():
    trace.append('Starting g2')
    yield 'g2 spam'
    yield 'g2 more spam'
    trace.append('Finishing g2')
for x in g1():
    trace.append('Yielded %s' % (x,))

assert trace == ['Starting g1', 'Yielded g1 ham', 'Starting g2', 'Yielded g2 spam', 'Yielded g2 more spam', 'Finishing g2', 'Yielded g1 eggs', 'Finishing g1']
print("TestPEP380Operation::test_delegation_of_next_call_to_subgenerator: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_delegation_of_next_call_to_subgenerator: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_delegation_of_next_to_non_generator.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_delegation_of_next_to_non_generator() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_delegation_of_next_to_non_generator"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_delegation_of_next_to_non_generator"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_delegation_of_next_to_non_generator
"""Auto-ported test: TestPEP380Operation::test_delegation_of_next_to_non_generator (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test delegation of next() to non-generator
        """
trace = []

def g():
    yield from range(3)
for x in g():
    trace.append('Yielded %s' % (x,))

assert trace == ['Yielded 0', 'Yielded 1', 'Yielded 2']
print("TestPEP380Operation::test_delegation_of_next_to_non_generator: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_delegation_of_next_to_non_generator: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_delegation_of_send.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_delegation_of_send() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_delegation_of_send"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_delegation_of_send"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_delegation_of_send
"""Auto-ported test: TestPEP380Operation::test_delegation_of_send (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test delegation of send()
        """
trace = []

def g1():
    trace.append('Starting g1')
    x = (yield 'g1 ham')
    trace.append('g1 received %s' % (x,))
    yield from g2()
    x = (yield 'g1 eggs')
    trace.append('g1 received %s' % (x,))
    trace.append('Finishing g1')

def g2():
    trace.append('Starting g2')
    x = (yield 'g2 spam')
    trace.append('g2 received %s' % (x,))
    x = (yield 'g2 more spam')
    trace.append('g2 received %s' % (x,))
    trace.append('Finishing g2')
g = g1()
y = next(g)
x = 1
try:
    while 1:
        y = g.send(x)
        trace.append('Yielded %s' % (y,))
        x += 1
except StopIteration:
    pass

assert trace == ['Starting g1', 'g1 received 1', 'Starting g2', 'Yielded g2 spam', 'g2 received 2', 'Yielded g2 more spam', 'g2 received 3', 'Finishing g2', 'Yielded g1 eggs', 'g1 received 4', 'Finishing g1']
print("TestPEP380Operation::test_delegation_of_send: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_delegation_of_send: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_exception_in_initial_next_call.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_exception_in_initial_next_call() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_exception_in_initial_next_call"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_exception_in_initial_next_call"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_exception_in_initial_next_call
"""Auto-ported test: TestPEP380Operation::test_exception_in_initial_next_call (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test exception in initial next() call
        """
trace = []

def g1():
    trace.append('g1 about to yield from g2')
    yield from g2()
    trace.append('g1 should not be here')

def g2():
    yield (1 / 0)

def run():
    gi = g1()
    next(gi)

try:
    run()
    raise AssertionError('expected ZeroDivisionError')
except ZeroDivisionError:
    pass

assert trace == ['g1 about to yield from g2']
print("TestPEP380Operation::test_exception_in_initial_next_call: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_exception_in_initial_next_call: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_exception_value_crash.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_exception_value_crash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_exception_value_crash"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_exception_value_crash"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_exception_value_crash
"""Auto-ported test: TestPEP380Operation::test_exception_value_crash (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
def g1():
    yield from g2()

def g2():
    yield 'g2'
    return [42]

assert list(g1()) == ['g2']
print("TestPEP380Operation::test_exception_value_crash: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_exception_value_crash: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_generator_return_value.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_generator_return_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_generator_return_value"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_generator_return_value"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_generator_return_value
"""Auto-ported test: TestPEP380Operation::test_generator_return_value (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test generator return value
        """
trace = []

def g1():
    trace.append('Starting g1')
    yield 'g1 ham'
    ret = (yield from g2())
    trace.append('g2 returned %r' % (ret,))
    for v in (1, (2,), StopIteration(3)):
        ret = (yield from g2(v))
        trace.append('g2 returned %r' % (ret,))
    yield 'g1 eggs'
    trace.append('Finishing g1')

def g2(v=None):
    trace.append('Starting g2')
    yield 'g2 spam'
    yield 'g2 more spam'
    trace.append('Finishing g2')
    if v:
        return v
for x in g1():
    trace.append('Yielded %s' % (x,))

assert trace == ['Starting g1', 'Yielded g1 ham', 'Starting g2', 'Yielded g2 spam', 'Yielded g2 more spam', 'Finishing g2', 'g2 returned None', 'Starting g2', 'Yielded g2 spam', 'Yielded g2 more spam', 'Finishing g2', 'g2 returned 1', 'Starting g2', 'Yielded g2 spam', 'Yielded g2 more spam', 'Finishing g2', 'g2 returned (2,)', 'Starting g2', 'Yielded g2 spam', 'Yielded g2 more spam', 'Finishing g2', 'g2 returned StopIteration(3)', 'Yielded g1 eggs', 'Finishing g1']
print("TestPEP380Operation::test_generator_return_value: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_generator_return_value: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_returning_value_from_delegated_throw.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_returning_value_from_delegated_throw() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_returning_value_from_delegated_throw"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_returning_value_from_delegated_throw"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_returning_value_from_delegated_throw
"""Auto-ported test: TestPEP380Operation::test_returning_value_from_delegated_throw (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test returning value from delegated 'throw'
        """
trace = []

def g1():
    try:
        trace.append('Starting g1')
        yield 'g1 ham'
        yield from g2()
        yield 'g1 eggs'
    finally:
        trace.append('Finishing g1')

def g2():
    try:
        trace.append('Starting g2')
        yield 'g2 spam'
        yield 'g2 more spam'
    except LunchError:
        trace.append('Caught LunchError in g2')
        yield 'g2 lunch saved'
        yield 'g2 yet more spam'

class LunchError(Exception):
    pass
g = g1()
for i in range(2):
    x = next(g)
    trace.append('Yielded %s' % (x,))
e = LunchError('tomato ejected')
g.throw(e)
for x in g:
    trace.append('Yielded %s' % (x,))

assert trace == ['Starting g1', 'Yielded g1 ham', 'Starting g2', 'Yielded g2 spam', 'Caught LunchError in g2', 'Yielded g2 yet more spam', 'Yielded g1 eggs', 'Finishing g1']
print("TestPEP380Operation::test_returning_value_from_delegated_throw: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_returning_value_from_delegated_throw: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_throwing_generator_exit_into_subgen_that_raises.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_throwing_generator_exit_into_subgen_that_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_throwing_generator_exit_into_subgen_that_raises"
# subject = "cpython.test_yield_from.TestPEP380Operation.test_throwing_GeneratorExit_into_subgen_that_raises"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_throwing_GeneratorExit_into_subgen_that_raises
"""Auto-ported test: TestPEP380Operation::test_throwing_GeneratorExit_into_subgen_that_raises (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
"""
        Test throwing GeneratorExit into a subgenerator that
        catches it and raises a different exception.
        """
trace = []

def f():
    try:
        trace.append('Enter f')
        yield
        trace.append('Exit f')
    except GeneratorExit:
        raise ValueError('Vorpal bunny encountered')

def g():
    trace.append('Enter g')
    yield from f()
    trace.append('Exit g')
try:
    gi = g()
    next(gi)
    gi.throw(GeneratorExit)
except ValueError as e:

    assert e.args[0] == 'Vorpal bunny encountered'

    assert isinstance(e.__context__, GeneratorExit)
else:

    raise AssertionError('subgenerator failed to raise ValueError')

assert trace == ['Enter g', 'Enter f']
print("TestPEP380Operation::test_throwing_GeneratorExit_into_subgen_that_raises: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_throwing_GeneratorExit_into_subgen_that_raises: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/yield_from/test_pep380_operation__test_yield_from_empty.py`.
#[test]
fn test_gen_behavior_core_yield_from_test_pep380_operation__test_yield_from_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "yield_from"
# dimension = "behavior"
# case = "test_pep380_operation__test_yield_from_empty"
# subject = "cpython.test.test_yield_from.TestPEP380Operation.test_yield_from_empty"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_yield_from.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_yield_from.py::TestPEP380Operation::test_yield_from_empty
"""Auto-ported test: TestPEP380Operation::test_yield_from_empty (CPython 3.12 oracle)."""


import unittest
import inspect
from test.support import captured_stderr, disable_gc, gc_collect
from test import support


'\nTest suite for PEP 380 implementation\n\nadapted from original tests written by Greg Ewing\nsee <http://www.cosc.canterbury.ac.nz/greg.ewing/python/yield-from/YieldFrom-Python3.1.2-rev5.zip>\n'


# --- test body ---
def g():
    yield from ()

try:
    next(g())
    raise AssertionError('expected StopIteration')
except StopIteration:
    pass
print("TestPEP380Operation::test_yield_from_empty: ok")
"###);
    assert_output(&out, r###"TestPEP380Operation::test_yield_from_empty: ok
"###);
}
