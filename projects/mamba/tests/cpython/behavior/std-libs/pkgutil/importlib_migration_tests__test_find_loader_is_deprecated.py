# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "importlib_migration_tests__test_find_loader_is_deprecated"
# subject = "cpython.test_pkgutil.ImportlibMigrationTests.test_find_loader_is_deprecated"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::ImportlibMigrationTests::test_find_loader_is_deprecated
"""Auto-ported test: ImportlibMigrationTests::test_find_loader_is_deprecated (CPython 3.12 oracle)."""


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
with check_warnings(('.*\\bpkgutil.find_loader\\b.*', DeprecationWarning)):
    res = pkgutil.find_loader('sys')

assert res is not None
print("ImportlibMigrationTests::test_find_loader_is_deprecated: ok")
