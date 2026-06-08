# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dynamic"
# dimension = "behavior"
# case = "test_tracing__test_after_specialization"
# subject = "cpython.test_dynamic.TestTracing.test_after_specialization"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dynamic.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_dynamic.py::TestTracing::test_after_specialization
"""Auto-ported test: TestTracing::test_after_specialization (CPython 3.12 oracle)."""


import builtins
import sys
import unittest
from test.support import swap_item, swap_attr


# --- test body ---
pass
sys.settrace(None)

def trace(frame, event, arg):
    return trace
turn_on_trace = False

class C:

    def __init__(self, x):
        self.x = x

    def __del__(self):
        if turn_on_trace:
            sys.settrace(trace)

def f():
    (C(0).x, len)

def g():
    [0][C(0).x]

def h():
    0 + C(0).x
for func in (f, g, h):
    for _ in range(58):
        func()
    turn_on_trace = True
    func()
    sys.settrace(None)
    turn_on_trace = False
print("TestTracing::test_after_specialization: ok")
