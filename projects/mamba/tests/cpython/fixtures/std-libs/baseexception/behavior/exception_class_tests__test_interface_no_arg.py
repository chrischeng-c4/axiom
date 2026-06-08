# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "baseexception"
# dimension = "behavior"
# case = "exception_class_tests__test_interface_no_arg"
# subject = "cpython.test_baseexception.ExceptionClassTests.test_interface_no_arg"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_baseexception.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_baseexception.py::ExceptionClassTests::test_interface_no_arg
"""Auto-ported test: ExceptionClassTests::test_interface_no_arg (CPython 3.12 oracle)."""


import unittest
import builtins
import os
from platform import system as platform_system


# --- test body ---
interface_tests = ('length', 'args', 'str', 'repr')

def interface_test_driver(results):
    for test_name, (given, expected) in zip(interface_tests, results):

        assert given == expected

def verify_instance_interface(ins):
    for attr in ('args', '__str__', '__repr__'):

        assert hasattr(ins, attr)
exc = Exception()
results = ([len(exc.args), 0], [exc.args, tuple()], [str(exc), ''], [repr(exc), exc.__class__.__name__ + '()'])
interface_test_driver(results)
print("ExceptionClassTests::test_interface_no_arg: ok")
