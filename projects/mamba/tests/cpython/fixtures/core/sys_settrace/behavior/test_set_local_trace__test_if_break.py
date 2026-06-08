# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "sys_settrace"
# dimension = "behavior"
# case = "test_set_local_trace__test_if_break"
# subject = "cpython.test.test_sys_settrace.TestSetLocalTrace.test_if_break"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys_settrace.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys_settrace.py::TestSetLocalTrace::test_if_break
"""Auto-ported test: TestSetLocalTrace::test_if_break (CPython 3.12 oracle)."""


from test import support
import unittest
import sys
import difflib
import gc
from functools import wraps
import asyncio
from test.support import import_helper
import contextlib
import warnings


support.requires_working_socket(module=True)

class tracecontext:
    """Context manager that traces its enter and exit."""

    def __init__(self, output, value):
        self.output = output
        self.value = value

    def __enter__(self):
        self.output.append(self.value)

    def __exit__(self, *exc_info):
        self.output.append(-self.value)

class asynctracecontext:
    """Asynchronous context manager that traces its aenter and aexit."""

    def __init__(self, output, value):
        self.output = output
        self.value = value

    async def __aenter__(self):
        self.output.append(self.value)

    async def __aexit__(self, *exc_info):
        self.output.append(-self.value)

async def asynciter(iterable):
    """Convert an iterable to an asynchronous iterator."""
    for x in iterable:
        yield x

def clean_asynciter(test):

    @wraps(test)
    async def wrapper(*args, **kwargs):
        cleanups = []

        def wrapped_asynciter(iterable):
            it = asynciter(iterable)
            cleanups.append(it.aclose)
            return it
        try:
            return await test(*args, **kwargs, asynciter=wrapped_asynciter)
        finally:
            while cleanups:
                await cleanups.pop()()
    return wrapper

def basic():
    return 1

basic.events = [(0, 'call'), (1, 'line'), (1, 'return')]

def arigo_example0():
    x = 1
    del x
    while 0:
        pass
    x = 1

arigo_example0.events = [(0, 'call'), (1, 'line'), (2, 'line'), (3, 'line'), (5, 'line'), (5, 'return')]

def arigo_example1():
    x = 1
    del x
    if 0:
        pass
    x = 1

arigo_example1.events = [(0, 'call'), (1, 'line'), (2, 'line'), (3, 'line'), (5, 'line'), (5, 'return')]

def arigo_example2():
    x = 1
    del x
    if 1:
        x = 1
    else:
        pass
    return None

arigo_example2.events = [(0, 'call'), (1, 'line'), (2, 'line'), (3, 'line'), (4, 'line'), (7, 'line'), (7, 'return')]

def one_instr_line():
    x = 1
    del x
    x = 1

one_instr_line.events = [(0, 'call'), (1, 'line'), (2, 'line'), (3, 'line'), (3, 'return')]

def no_pop_tops():
    x = 1
    for a in range(2):
        if a:
            x = 1
        else:
            x = 1

no_pop_tops.events = [(0, 'call'), (1, 'line'), (2, 'line'), (3, 'line'), (6, 'line'), (2, 'line'), (3, 'line'), (4, 'line'), (2, 'line'), (2, 'return')]

def no_pop_blocks():
    y = 1
    while not y:
        bla
    x = 1

no_pop_blocks.events = [(0, 'call'), (1, 'line'), (2, 'line'), (4, 'line'), (4, 'return')]

def called():
    x = 1

def call():
    called()

call.events = [(0, 'call'), (1, 'line'), (-3, 'call'), (-2, 'line'), (-2, 'return'), (1, 'return')]

def raises():
    raise Exception

def test_raise():
    try:
        raises()
    except Exception:
        pass

test_raise.events = [(0, 'call'), (1, 'line'), (2, 'line'), (-3, 'call'), (-2, 'line'), (-2, 'exception'), (-2, 'return'), (2, 'exception'), (3, 'line'), (4, 'line'), (4, 'return')]

def _settrace_and_return(tracefunc):
    sys.settrace(tracefunc)
    sys._getframe().f_back.f_trace = tracefunc

def settrace_and_return(tracefunc):
    _settrace_and_return(tracefunc)

settrace_and_return.events = [(1, 'return')]

def _settrace_and_raise(tracefunc):
    sys.settrace(tracefunc)
    sys._getframe().f_back.f_trace = tracefunc
    raise RuntimeError

def settrace_and_raise(tracefunc):
    try:
        _settrace_and_raise(tracefunc)
    except RuntimeError:
        pass

settrace_and_raise.events = [(2, 'exception'), (3, 'line'), (4, 'line'), (4, 'return')]

def ireturn_example():
    a = 5
    b = 5
    if a == b:
        b = a + 1
    else:
        pass

ireturn_example.events = [(0, 'call'), (1, 'line'), (2, 'line'), (3, 'line'), (4, 'line'), (4, 'return')]

def tightloop_example():
    items = range(0, 3)
    try:
        i = 0
        while 1:
            b = items[i]
            i += 1
    except IndexError:
        pass

tightloop_example.events = [(0, 'call'), (1, 'line'), (2, 'line'), (3, 'line'), (4, 'line'), (5, 'line'), (4, 'line'), (5, 'line'), (4, 'line'), (5, 'line'), (4, 'line'), (5, 'line'), (5, 'exception'), (6, 'line'), (7, 'line'), (7, 'return')]

def tighterloop_example():
    items = range(1, 4)
    try:
        i = 0
        while 1:
            i = items[i]
    except IndexError:
        pass

tighterloop_example.events = [(0, 'call'), (1, 'line'), (2, 'line'), (3, 'line'), (4, 'line'), (4, 'line'), (4, 'line'), (4, 'line'), (4, 'exception'), (5, 'line'), (6, 'line'), (6, 'return')]

def generator_function():
    try:
        yield True
        'continued'
    finally:
        'finally'

def generator_example():
    x = any(generator_function())
    for x in range(10):
        y = x

generator_example.events = [(0, 'call'), (2, 'line'), (-6, 'call'), (-5, 'line'), (-4, 'line'), (-4, 'return'), (-4, 'call'), (-4, 'exception'), (-1, 'line'), (-1, 'return')] + [(5, 'line'), (6, 'line')] * 10 + [(5, 'line'), (5, 'return')]

class Tracer:

    def __init__(self, trace_line_events=None, trace_opcode_events=None):
        self.trace_line_events = trace_line_events
        self.trace_opcode_events = trace_opcode_events
        self.events = []

    def _reconfigure_frame(self, frame):
        if self.trace_line_events is not None:
            frame.f_trace_lines = self.trace_line_events
        if self.trace_opcode_events is not None:
            frame.f_trace_opcodes = self.trace_opcode_events

    def trace(self, frame, event, arg):
        self._reconfigure_frame(frame)
        self.events.append((frame.f_lineno, event))
        return self.trace

    def traceWithGenexp(self, frame, event, arg):
        self._reconfigure_frame(frame)
        (o for o in [1])
        self.events.append((frame.f_lineno, event))
        return self.trace

EVENT_NAMES = ['call', 'exception', 'line', 'return']

class JumpTracer:
    """Defines a trace function that jumps from one place to another."""

    def __init__(self, function, jumpFrom, jumpTo, event='line', decorated=False):
        self.code = function.__code__
        self.jumpFrom = jumpFrom
        self.jumpTo = jumpTo
        self.event = event
        self.firstLine = None if decorated else self.code.co_firstlineno
        self.done = False

    def trace(self, frame, event, arg):
        if self.done:
            return
        if self.firstLine is None and frame.f_code == self.code and (event == 'line'):
            self.firstLine = frame.f_lineno - 1
        if event == self.event and self.firstLine is not None and (frame.f_lineno == self.firstLine + self.jumpFrom):
            f = frame
            while f is not None and f.f_code != self.code:
                f = f.f_back
            if f is not None:
                try:
                    frame.f_lineno = self.firstLine + self.jumpTo
                except TypeError:
                    frame.f_lineno = self.jumpTo
                self.done = True
        return self.trace

def no_jump_to_non_integers(output):
    try:
        output.append(2)
    except ValueError as e:
        output.append('integer' in str(e))

def no_jump_without_trace_function():
    try:
        previous_frame = sys._getframe().f_back
        previous_frame.f_lineno = previous_frame.f_lineno
    except ValueError as e:
        if 'trace' not in str(e):
            raise
    else:
        raise AssertionError('Trace-function-less jump failed to fail')


# --- test body ---
def compare_events(line_offset, events, expected_events):
    events = [(l - line_offset if l is not None else None, e) for l, e in events]
    if events != expected_events:

        raise AssertionError('events did not match expectation:\n' + '\n'.join(difflib.ndiff([str(x) for x in expected_events], [str(x) for x in events])))

def make_tracer():
    """Helper to allow test subclasses to configure tracers differently"""
    return Tracer()

def run_and_compare(func, events):
    tracer = make_tracer()
    sys.settrace(tracer.trace)
    func()
    sys.settrace(None)
    compare_events(func.__code__.co_firstlineno, tracer.events, events)

def run_test(func):
    run_and_compare(func, func.events)

def run_test2(func):
    tracer = make_tracer()
    func(tracer.trace)
    sys.settrace(None)
    compare_events(func.__code__.co_firstlineno, tracer.events, func.events)
self_using_gc = gc.isenabled()
gc.disable()
pass

def func():
    seq = [1, 0]
    while seq:
        n = seq.pop()
        if n:
            break
    else:
        n = 99
    return n
run_and_compare(func, [(0, 'call'), (1, 'line'), (2, 'line'), (3, 'line'), (4, 'line'), (2, 'line'), (3, 'line'), (4, 'line'), (5, 'line'), (8, 'line'), (8, 'return')])
print("TestSetLocalTrace::test_if_break: ok")
