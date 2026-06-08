# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "baseexception"
# dimension = "behavior"
# case = "usage_tests__test_catch_base_exception_instance"
# subject = "cpython.test_baseexception.UsageTests.test_catch_BaseException_instance"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_baseexception.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_baseexception.py::UsageTests::test_catch_BaseException_instance
"""Auto-ported test: UsageTests::test_catch_BaseException_instance (CPython 3.12 oracle)."""


import unittest
import builtins
import os
from platform import system as platform_system


# --- test body ---
def catch_fails(object_):
    """Catching 'object_' should raise a TypeError."""
    try:
        try:
            raise Exception
        except object_:
            pass
    except TypeError:
        pass
    except Exception:

        raise AssertionError('TypeError expected when catching %s' % type(object_))
    try:
        try:
            raise Exception
        except (object_,):
            pass
    except TypeError:
        return
    except Exception:

        raise AssertionError('TypeError expected when catching %s as specified in a tuple' % type(object_))

def raise_fails(object_):
    """Make sure that raising 'object_' triggers a TypeError."""
    try:
        raise object_
    except TypeError:
        return

    raise AssertionError('TypeError expected for raising %s' % type(object_))
catch_fails(BaseException())
print("UsageTests::test_catch_BaseException_instance: ok")
