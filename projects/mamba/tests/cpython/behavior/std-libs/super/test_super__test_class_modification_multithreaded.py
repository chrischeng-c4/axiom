# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "super"
# dimension = "behavior"
# case = "test_super__test_class_modification_multithreaded"
# subject = "cpython.test_super.TestSuper.test___class___modification_multithreaded"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_super.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_super.py::TestSuper::test___class___modification_multithreaded
"""Auto-ported test: TestSuper::test___class___modification_multithreaded (CPython 3.12 oracle)."""


import textwrap
import threading
import unittest
from unittest.mock import patch
from test.support import import_helper, threading_helper


'Unit tests for zero-argument super() & related machinery.'

ADAPTIVE_WARMUP_DELAY = 2

class A:

    def f(self):
        return 'A'

    @classmethod
    def cm(cls):
        return (cls, 'A')

class B(A):

    def f(self):
        return super().f() + 'B'

    @classmethod
    def cm(cls):
        return (cls, super().cm(), 'B')

class C(A):

    def f(self):
        return super().f() + 'C'

    @classmethod
    def cm(cls):
        return (cls, super().cm(), 'C')

class D(C, B):

    def f(self):
        return super().f() + 'D'

    def cm(cls):
        return (cls, super().cm(), 'D')

class E(D):
    pass

class F(E):
    f = E.f

class G(A):
    pass


# --- test body ---
""" Note: this test isn't actually testing anything on its own.
        It requires a sys audithook to be set to crash on older Python.
        This should be the case anyways as our test suite sets
        an audit hook.
        """

class Foo:
    pass

class Bar:
    pass
thing = Foo()

def work():
    foo = thing
    for _ in range(5000):
        foo.__class__ = Bar
        type(foo)
        foo.__class__ = Foo
        type(foo)
threads = []
for _ in range(6):
    thread = threading.Thread(target=work)
    thread.start()
    threads.append(thread)
for thread in threads:
    thread.join()
print("TestSuper::test___class___modification_multithreaded: ok")
