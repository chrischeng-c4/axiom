# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys_setprofile"
# dimension = "behavior"
# case = "profile_hook_test_case__test_raise_reraise"
# subject = "cpython.test_sys_setprofile.ProfileHookTestCase.test_raise_reraise"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys_setprofile.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys_setprofile.py::ProfileHookTestCase::test_raise_reraise
"""Auto-ported test: ProfileHookTestCase::test_raise_reraise (CPython 3.12 oracle)."""


import gc
import pprint
import sys
import unittest


class HookWatcher:

    def __init__(self):
        self.frames = []
        self.events = []

    def callback(self, frame, event, arg):
        if event == 'call' or event == 'return' or event == 'exception':
            self.add_event(event, frame)

    def add_event(self, event, frame=None):
        """Add an event to the log."""
        if frame is None:
            frame = sys._getframe(1)
        try:
            frameno = self.frames.index(frame)
        except ValueError:
            frameno = len(self.frames)
            self.frames.append(frame)
        self.events.append((frameno, event, ident(frame)))

    def get_events(self):
        """Remove calls to add_event()."""
        disallowed = [ident(self.add_event.__func__), ident(ident)]
        self.frames = None
        return [item for item in self.events if item[2] not in disallowed]

class ProfileSimulator(HookWatcher):

    def __init__(self, testcase):
        self.testcase = testcase
        self.stack = []
        HookWatcher.__init__(self)

    def callback(self, frame, event, arg):
        self.dispatch[event](self, frame)

    def trace_call(self, frame):
        self.add_event('call', frame)
        self.stack.append(frame)

    def trace_return(self, frame):
        self.add_event('return', frame)
        self.stack.pop()

    def trace_exception(self, frame):
        self.testcase.fail('the profiler should never receive exception events')

    def trace_pass(self, frame):
        pass
    dispatch = {'call': trace_call, 'exception': trace_exception, 'return': trace_return, 'c_call': trace_pass, 'c_return': trace_pass, 'c_exception': trace_pass}

def ident(function):
    if hasattr(function, 'f_code'):
        code = function.f_code
    else:
        code = function.__code__
    return (code.co_firstlineno, code.co_name)

def protect(f, p):
    try:
        f(p)
    except:
        pass

protect_ident = ident(protect)

def capture_events(callable, p=None):
    if p is None:
        p = HookWatcher()
    old_gc = gc.isenabled()
    gc.disable()
    try:
        sys.setprofile(p.callback)
        protect(callable, p)
        sys.setprofile(None)
    finally:
        if old_gc:
            gc.enable()
    return p.get_events()[1:-1]

def show_events(callable):
    import pprint
    pprint.pprint(capture_events(callable))


# --- test body ---
def check_events(callable, expected):
    events = capture_events(callable, new_watcher())
    if events != expected:

        raise AssertionError('Expected events:\n%s\nReceived events:\n%s' % (pprint.pformat(expected), pprint.pformat(events)))

def new_watcher():
    return HookWatcher()

def f(p):
    try:
        1 / 0
    except:
        raise
f_ident = ident(f)
check_events(f, [(1, 'call', f_ident), (1, 'return', f_ident)])
print("ProfileHookTestCase::test_raise_reraise: ok")
