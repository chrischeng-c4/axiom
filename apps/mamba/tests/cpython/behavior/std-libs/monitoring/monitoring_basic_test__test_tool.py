# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "monitoring"
# dimension = "behavior"
# case = "monitoring_basic_test__test_tool"
# subject = "cpython.test_monitoring.MonitoringBasicTest.test_tool"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_monitoring.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_monitoring.py::MonitoringBasicTest::test_tool
"""Auto-ported test: MonitoringBasicTest::test_tool (CPython 3.12 oracle)."""


import collections
import dis
import functools
import operator
import sys
import textwrap
import types
import unittest
import asyncio
from test.profilee import testfunc


'Test suite for the sys.monitoring.'

PAIR = (0, 1)

def f1():
    pass

def f2():
    len([])
    sys.getsizeof(0)

def floop():
    for item in PAIR:
        pass

def gen():
    yield
    yield

def g1():
    for _ in gen():
        pass

TEST_TOOL = 2

TEST_TOOL2 = 3

TEST_TOOL3 = 4

class MonitoringTestBase:

    def setUp(self):
        for tool in range(6):
            self.assertEqual(sys.monitoring.get_events(tool), 0)
        self.assertIs(sys.monitoring.get_tool(TEST_TOOL), None)
        self.assertIs(sys.monitoring.get_tool(TEST_TOOL2), None)
        self.assertIs(sys.monitoring.get_tool(TEST_TOOL3), None)
        sys.monitoring.use_tool_id(TEST_TOOL, 'test ' + self.__class__.__name__)
        sys.monitoring.use_tool_id(TEST_TOOL2, 'test2 ' + self.__class__.__name__)
        sys.monitoring.use_tool_id(TEST_TOOL3, 'test3 ' + self.__class__.__name__)

    def tearDown(self):
        for tool in range(6):
            self.assertEqual(sys.monitoring.get_events(tool), 0)
        sys.monitoring.free_tool_id(TEST_TOOL)
        sys.monitoring.free_tool_id(TEST_TOOL2)
        sys.monitoring.free_tool_id(TEST_TOOL3)

E = sys.monitoring.events

INSTRUMENTED_EVENTS = [(E.PY_START, 'start'), (E.PY_RESUME, 'resume'), (E.PY_RETURN, 'return'), (E.PY_YIELD, 'yield'), (E.JUMP, 'jump'), (E.BRANCH, 'branch')]

EXCEPT_EVENTS = [(E.RAISE, 'raise'), (E.PY_UNWIND, 'unwind'), (E.EXCEPTION_HANDLED, 'exception_handled')]

SIMPLE_EVENTS = INSTRUMENTED_EVENTS + EXCEPT_EVENTS + [(E.C_RAISE, 'c_raise'), (E.C_RETURN, 'c_return')]

SIMPLE_EVENT_SET = functools.reduce(operator.or_, [ev for ev, _ in SIMPLE_EVENTS], 0) | E.CALL

def just_pass():
    pass

just_pass.events = ['py_call', 'start', 'return']

def just_raise():
    raise Exception

just_raise.events = ['py_call', 'start', 'raise', 'unwind']

def just_call():
    len([])

just_call.events = ['py_call', 'start', 'c_call', 'c_return', 'return']

def caught():
    try:
        1 / 0
    except Exception:
        pass

caught.events = ['py_call', 'start', 'raise', 'exception_handled', 'branch', 'return']

def nested_call():
    just_pass()

nested_call.events = ['py_call', 'start', 'py_call', 'start', 'return', 'return']

PY_CALLABLES = (types.FunctionType, types.MethodType)

class MonitoringEventsBase(MonitoringTestBase):

    def gather_events(self, func):
        events = []
        for event, event_name in SIMPLE_EVENTS:

            def record(*args, event_name=event_name):
                events.append(event_name)
            sys.monitoring.register_callback(TEST_TOOL, event, record)

        def record_call(code, offset, obj, arg):
            if isinstance(obj, PY_CALLABLES):
                events.append('py_call')
            else:
                events.append('c_call')
        sys.monitoring.register_callback(TEST_TOOL, E.CALL, record_call)
        sys.monitoring.set_events(TEST_TOOL, SIMPLE_EVENT_SET)
        events = []
        try:
            func()
        except:
            pass
        sys.monitoring.set_events(TEST_TOOL, 0)
        events = events[:-1]
        return events

    def check_events(self, func, expected=None):
        events = self.gather_events(func)
        if expected is None:
            expected = func.events
        self.assertEqual(events, expected)

UP_EVENTS = (E.C_RETURN, E.C_RAISE, E.PY_RETURN, E.PY_UNWIND, E.PY_YIELD)

DOWN_EVENTS = (E.PY_START, E.PY_RESUME)

class CounterWithDisable:

    def __init__(self):
        self.disable = False
        self.count = 0

    def __call__(self, *args):
        self.count += 1
        if self.disable:
            return sys.monitoring.DISABLE

class RecorderWithDisable:

    def __init__(self, events):
        self.disable = False
        self.events = events

    def __call__(self, code, event):
        self.events.append(event)
        if self.disable:
            return sys.monitoring.DISABLE

class ExceptionRecorder:
    event_type = E.RAISE

    def __init__(self, events):
        self.events = events

    def __call__(self, code, offset, exc):
        self.events.append(('raise', type(exc)))

class CheckEvents(MonitoringTestBase, unittest.TestCase):

    def get_events(self, func, tool, recorders):
        try:
            self.assertEqual(sys.monitoring._all_events(), {})
            event_list = []
            all_events = 0
            for recorder in recorders:
                ev = recorder.event_type
                sys.monitoring.register_callback(tool, ev, recorder(event_list))
                all_events |= ev
            sys.monitoring.set_events(tool, all_events)
            func()
            sys.monitoring.set_events(tool, 0)
            for recorder in recorders:
                sys.monitoring.register_callback(tool, recorder.event_type, None)
            return event_list
        finally:
            sys.monitoring.set_events(tool, 0)
            for recorder in recorders:
                sys.monitoring.register_callback(tool, recorder.event_type, None)

    def check_events(self, func, expected, tool=TEST_TOOL, recorders=(ExceptionRecorder,)):
        events = self.get_events(func, tool, recorders)
        if events != expected:
            print(events, file=sys.stderr)
        self.assertEqual(events, expected)

    def check_balanced(self, func, recorders):
        events = self.get_events(func, TEST_TOOL, recorders)
        self.assertEqual(len(events) % 2, 0)
        for r, h in zip(events[::2], events[1::2]):
            r0 = r[0]
            self.assertIn(r0, ('raise', 'reraise'))
            h0 = h[0]
            self.assertIn(h0, ('handled', 'unwind'))
            self.assertEqual(r[1], h[1])

class StopiterationRecorder(ExceptionRecorder):
    event_type = E.STOP_ITERATION

class ReraiseRecorder(ExceptionRecorder):
    event_type = E.RERAISE

    def __call__(self, code, offset, exc):
        self.events.append(('reraise', type(exc)))

class UnwindRecorder(ExceptionRecorder):
    event_type = E.PY_UNWIND

    def __call__(self, code, offset, exc):
        self.events.append(('unwind', type(exc)))

class ExceptionHandledRecorder(ExceptionRecorder):
    event_type = E.EXCEPTION_HANDLED

    def __call__(self, code, offset, exc):
        self.events.append(('handled', type(exc)))

class ThrowRecorder(ExceptionRecorder):
    event_type = E.PY_THROW

    def __call__(self, code, offset, exc):
        self.events.append(('throw', type(exc)))

class LineRecorder:
    event_type = E.LINE

    def __init__(self, events):
        self.events = events

    def __call__(self, code, line):
        self.events.append(('line', code.co_name, line - code.co_firstlineno))

class CallRecorder:
    event_type = E.CALL

    def __init__(self, events):
        self.events = events

    def __call__(self, code, offset, func, arg):
        self.events.append(('call', func.__name__, arg))

class CEventRecorder:

    def __init__(self, events):
        self.events = events

    def __call__(self, code, offset, func, arg):
        self.events.append((self.event_name, func.__name__, arg))

class CReturnRecorder(CEventRecorder):
    event_type = E.C_RETURN
    event_name = 'C return'

class CRaiseRecorder(CEventRecorder):
    event_type = E.C_RAISE
    event_name = 'C raise'

MANY_RECORDERS = (ExceptionRecorder, CallRecorder, LineRecorder, CReturnRecorder, CRaiseRecorder)

class InstructionRecorder:
    event_type = E.INSTRUCTION

    def __init__(self, events):
        self.events = events

    def __call__(self, code, offset):
        if code.co_name != 'get_events':
            self.events.append(('instruction', code.co_name, offset))

LINE_AND_INSTRUCTION_RECORDERS = (InstructionRecorder, LineRecorder)

LOCAL_RECORDERS = (CallRecorder, LineRecorder, CReturnRecorder, CRaiseRecorder)

def line_from_offset(code, offset):
    for start, end, line in code.co_lines():
        if start <= offset < end:
            if line is None:
                return f'[offset={offset}]'
            return line - code.co_firstlineno
    return -1

class JumpRecorder:
    event_type = E.JUMP
    name = 'jump'

    def __init__(self, events):
        self.events = events

    def __call__(self, code, from_, to):
        from_line = line_from_offset(code, from_)
        to_line = line_from_offset(code, to)
        self.events.append((self.name, code.co_name, from_line, to_line))

class BranchRecorder(JumpRecorder):
    event_type = E.BRANCH
    name = 'branch'

class ReturnRecorder:
    event_type = E.PY_RETURN

    def __init__(self, events):
        self.events = events

    def __call__(self, code, offset, val):
        self.events.append(('return', val))

JUMP_AND_BRANCH_RECORDERS = (JumpRecorder, BranchRecorder)

JUMP_BRANCH_AND_LINE_RECORDERS = (JumpRecorder, BranchRecorder, LineRecorder)

FLOW_AND_LINE_RECORDERS = (JumpRecorder, BranchRecorder, LineRecorder, ExceptionRecorder, ReturnRecorder)


# --- test body ---
sys.monitoring.use_tool_id(TEST_TOOL, 'MonitoringTest.Tool')

assert sys.monitoring.get_tool(TEST_TOOL) == 'MonitoringTest.Tool'
sys.monitoring.set_events(TEST_TOOL, 15)

assert sys.monitoring.get_events(TEST_TOOL) == 15
sys.monitoring.set_events(TEST_TOOL, 0)
try:
    sys.monitoring.set_events(TEST_TOOL, sys.monitoring.events.C_RETURN)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
try:
    sys.monitoring.set_events(TEST_TOOL, sys.monitoring.events.C_RAISE)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
sys.monitoring.free_tool_id(TEST_TOOL)

assert sys.monitoring.get_tool(TEST_TOOL) == None
try:
    sys.monitoring.set_events(TEST_TOOL, sys.monitoring.events.CALL)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("MonitoringBasicTest::test_tool: ok")
