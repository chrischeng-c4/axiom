# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pkgutil"
# dimension = "behavior"
# case = "extend_path_tests__test_iter_importers"
# subject = "cpython.test_pkgutil.ExtendPathTests.test_iter_importers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pkgutil.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pkgutil.py::ExtendPathTests::test_iter_importers
"""Auto-ported test: ExtendPathTests::test_iter_importers (CPython 3.12 oracle)."""


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
iter_importers = pkgutil.iter_importers
get_importer = pkgutil.get_importer
pkgname = 'spam'
modname = 'eggs'
dirname = create_init(pkgname)
pathitem = os.path.join(dirname, pkgname)
fullname = '{}.{}'.format(pkgname, modname)
sys.modules.pop(fullname, None)
sys.modules.pop(pkgname, None)
try:
    create_submodule(dirname, pkgname, modname, 0)
    importlib.import_module(fullname)
    importers = list(iter_importers(fullname))
    expected_importer = get_importer(pathitem)
    for finder in importers:
        spec = finder.find_spec(fullname)
        loader = spec.loader
        try:
            loader = loader.loader
        except AttributeError:
            pass

        assert isinstance(finder, importlib.machinery.FileFinder)

        assert finder == expected_importer

        assert isinstance(loader, importlib.machinery.SourceFileLoader)

        assert finder.find_spec(pkgname) is None
    try:
        list(iter_importers('invalid.module'))
        raise AssertionError('expected ImportError')
    except ImportError:
        pass
    try:
        list(iter_importers('.spam'))
        raise AssertionError('expected ImportError')
    except ImportError:
        pass
finally:
    shutil.rmtree(dirname)
    del sys.path[0]
    try:
        del sys.modules['spam']
        del sys.modules['spam.eggs']
    except KeyError:
        pass
print("ExtendPathTests::test_iter_importers: ok")
