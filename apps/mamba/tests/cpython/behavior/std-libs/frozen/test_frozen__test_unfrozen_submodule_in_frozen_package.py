# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "frozen"
# dimension = "behavior"
# case = "test_frozen__test_unfrozen_submodule_in_frozen_package"
# subject = "cpython.test_frozen.TestFrozen.test_unfrozen_submodule_in_frozen_package"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_frozen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_frozen.py::TestFrozen::test_unfrozen_submodule_in_frozen_package
"""Auto-ported test: TestFrozen::test_unfrozen_submodule_in_frozen_package (CPython 3.12 oracle)."""


import importlib.machinery
import sys
import unittest
from test.support import captured_stdout, import_helper


'Basic test of the frozen module (source is in Python/frozen.c).'


# --- test body ---
with import_helper.CleanImport('__phello__', '__phello__.spam'):
    with import_helper.frozen_modules(enabled=True):
        import __phello__
    with import_helper.frozen_modules(enabled=False):
        import __phello__.spam as spam

assert spam is __phello__.spam

assert __phello__.__spec__.loader is importlib.machinery.FrozenImporter

assert spam.__spec__.loader is not importlib.machinery.FrozenImporter
print("TestFrozen::test_unfrozen_submodule_in_frozen_package: ok")
