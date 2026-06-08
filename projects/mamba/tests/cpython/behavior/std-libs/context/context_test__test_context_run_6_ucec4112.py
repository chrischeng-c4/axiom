# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "context"
# dimension = "behavior"
# case = "context_test__test_context_run_6_ucec4112"
# subject = "cpython.test_context.ContextTest.test_context_run_6"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_context.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import concurrent.futures
import contextvars
import functools
import gc
import random
import time
import weakref
ctx = contextvars.Context()
c = contextvars.ContextVar('a', default=0)

def fun():
    assert c.get() == 0
    assert ctx.get(c) is None
    c.set(42)
    assert c.get() == 42
    assert ctx.get(c) == 42
ctx.run(fun)

print("ContextTest::test_context_run_6: ok")
