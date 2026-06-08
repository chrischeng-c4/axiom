# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "extend_path_tests__test_simple"
# subject = "cpython.test_pkgutil.ExtendPathTests.test_simple"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::ExtendPathTests::test_simple
"""Auto-ported test: ExtendPathTests::test_simple (CPython 3.12 oracle)."""


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
def create_init(pkgname):
    dirname = tempfile.mkdtemp()
    sys.path.insert(0, dirname)
    pkgdir = os.path.join(dirname, pkgname)
    os.mkdir(pkgdir)
    with open(os.path.join(pkgdir, '__init__.py'), 'w') as fl:
        fl.write('from pkgutil import extend_path\n__path__ = extend_path(__path__, __name__)\n')
    return dirname

def create_submodule(dirname, pkgname, submodule_name, value):
    module_name = os.path.join(dirname, pkgname, submodule_name + '.py')
    with open(module_name, 'w') as fl:
        print('value={}'.format(value), file=fl)
pkgname = 'foo'
dirname_0 = create_init(pkgname)
dirname_1 = create_init(pkgname)
create_submodule(dirname_0, pkgname, 'bar', 0)
create_submodule(dirname_1, pkgname, 'baz', 1)
import foo.bar
import foo.baz

assert foo.bar.value == 0

assert foo.baz.value == 1

assert sorted(foo.__path__) == sorted([os.path.join(dirname_0, pkgname), os.path.join(dirname_1, pkgname)])
shutil.rmtree(dirname_0)
shutil.rmtree(dirname_1)
del sys.path[0]
del sys.path[0]
del sys.modules['foo']
del sys.modules['foo.bar']
del sys.modules['foo.baz']
print("ExtendPathTests::test_simple: ok")
