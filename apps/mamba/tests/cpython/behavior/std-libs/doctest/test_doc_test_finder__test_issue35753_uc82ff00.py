# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "doctest"
# dimension = "behavior"
# case = "test_doc_test_finder__test_issue35753_uc82ff00"
# subject = "cpython.test_doctest.TestDocTestFinder.test_issue35753"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_doctest/test_doctest.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import doctest
import functools
import os
import sys
import importlib
import importlib.abc
import importlib.util
import tempfile
import types
import contextlib
from unittest.mock import call
dummy_module = types.ModuleType('dummy')
dummy_module.__dict__['inject_call'] = call
finder = doctest.DocTestFinder()
assert finder.find(dummy_module) == []

print("TestDocTestFinder::test_issue35753: ok")
