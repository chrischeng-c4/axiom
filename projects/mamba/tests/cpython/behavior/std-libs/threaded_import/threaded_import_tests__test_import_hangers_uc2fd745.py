# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threaded_import"
# dimension = "behavior"
# case = "threaded_import_tests__test_import_hangers_uc2fd745"
# subject = "cpython.test_threaded_import.ThreadedImportTests.test_import_hangers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_threaded_import.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import _imp as imp
import os
import importlib
import sys
import time
import shutil
import threading
self_old_random = sys.modules.pop('random', None)
try:
    del sys.modules['test.test_importlib.threaded_import_hangers']
except KeyError:
    pass
import test.test_importlib.threaded_import_hangers
assert not test.test_importlib.threaded_import_hangers.errors

print("ThreadedImportTests::test_import_hangers: ok")
