# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "importlib_migration_tests__test_get_loader_handles_missing_spec_attribute"
# subject = "cpython.test_pkgutil.ImportlibMigrationTests.test_get_loader_handles_missing_spec_attribute"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::ImportlibMigrationTests::test_get_loader_handles_missing_spec_attribute
"""Auto-ported test: ImportlibMigrationTests::test_get_loader_handles_missing_spec_attribute (CPython 3.12 oracle)."""


from pathlib import Path
from test.support.import_helper import unload, CleanImport
from test.support.warnings_helper import check_warnings, ignore_warnings
import unittest
import sys
import importlib
from importlib.util import spec_from_file_location
import pkgutil
import os
import os.path
import tempfile
import shutil
import zipfile
from test.support.import_helper import DirsOnSysPath
from test.support.os_helper import FakePath
from test.test_importlib.util import uncache


def tearDownModule():
    import zipimport
    import importlib
    zipimport._zip_directory_cache.clear()
    importlib.invalidate_caches()


# --- test body ---
name = 'spam'
mod = type(sys)(name)
del mod.__spec__
with CleanImport(name):
    try:
        sys.modules[name] = mod
        loader = pkgutil.get_loader(name)
    finally:
        sys.modules.pop(name, None)

assert loader is None
print("ImportlibMigrationTests::test_get_loader_handles_missing_spec_attribute: ok")
