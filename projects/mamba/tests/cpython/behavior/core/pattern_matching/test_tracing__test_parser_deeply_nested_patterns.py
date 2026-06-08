# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_tracing__test_parser_deeply_nested_patterns"
# subject = "cpython.test_patma.TestTracing.test_parser_deeply_nested_patterns"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestTracing::test_parser_deeply_nested_patterns
"""Auto-ported test: TestTracing::test_parser_deeply_nested_patterns (CPython 3.12 oracle)."""


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
def _trace(func, *args, **kwargs):
    actual_linenos = []

    def trace(frame, event, arg):
        if event == 'line' and frame.f_code.co_name == func.__name__:
            assert arg is None
            relative_lineno = frame.f_lineno - func.__code__.co_firstlineno
            actual_linenos.append(relative_lineno)
        return trace
    old_trace = sys.gettrace()
    sys.settrace(trace)
    try:
        func(*args, **kwargs)
    finally:
        sys.settrace(old_trace)
    return actual_linenos
levels = 100
patterns = ['A' + '(' * levels + ')' * levels, '{1:' * levels + '1' + '}' * levels, '[' * levels + '1' + ']' * levels]
for pattern in patterns:
    code = inspect.cleandoc('\n                    match None:\n                        case {}:\n                            pass\n                '.format(pattern))
    compile(code, '<string>', 'exec')
print("TestTracing::test_parser_deeply_nested_patterns: ok")
