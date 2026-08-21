# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "context"
# dimension = "behavior"
# case = "context_test__test_context_copy_1_uce2ea35"
# subject = "cpython.test_context.ContextTest.test_context_copy_1"
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
ctx1 = contextvars.Context()
c = contextvars.ContextVar('c', default=42)

def ctx1_fun():
    c.set(10)
    ctx2 = ctx1.copy()
    assert ctx2[c] == 10
    c.set(20)
    assert ctx1[c] == 20
    assert ctx2[c] == 10
    ctx2.run(ctx2_fun)
    assert ctx1[c] == 20
    assert ctx2[c] == 30

def ctx2_fun():
    assert c.get() == 10
    c.set(30)
    assert c.get() == 30
ctx1.run(ctx1_fun)

print("ContextTest::test_context_copy_1: ok")
