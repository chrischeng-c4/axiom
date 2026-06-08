# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "import"
# dimension = "behavior"
# case = "subinterp_import_tests__test_single_init_extension_compat"
# subject = "cpython.__init__.SubinterpImportTests.test_single_init_extension_compat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_import/__init__.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 __init__.py::SubinterpImportTests::test_single_init_extension_compat
"""Auto-ported test: SubinterpImportTests::test_single_init_extension_compat (CPython 3.12 oracle)."""


import builtins
import errno
import glob
import json
import importlib.util
from importlib._bootstrap_external import _get_sourcefile
from importlib.machinery import BuiltinImporter, ExtensionFileLoader, FrozenImporter, SourceFileLoader
import marshal
import os
import py_compile
import random
import shutil
import stat
import subprocess
import sys
import textwrap
import threading
import time
import types
import unittest
from unittest import mock
import _testinternalcapi
import _imp
from test.support import os_helper
from test.support import STDLIB_DIR, swap_attr, swap_item, cpython_only, is_emscripten, is_wasi, run_in_subinterp, run_in_subinterp_with_config
from test.support.import_helper import forget, make_legacy_pyc, unlink, unload, ready_to_import, DirsOnSysPath, CleanImport
from test.support.os_helper import TESTFN, rmtree, temp_umask, TESTFN_UNENCODABLE
from test.support import script_helper
from test.support import threading_helper
from test.test_importlib.util import uncache
from types import ModuleType


try:
    import _testsinglephase
except ImportError:
    _testsinglephase = None

try:
    import _testmultiphase
except ImportError:
    _testmultiphase = None

try:
    import _xxsubinterpreters as _interpreters
except ModuleNotFoundError:
    _interpreters = None

skip_if_dont_write_bytecode = unittest.skipIf(sys.dont_write_bytecode, 'test meaningful only when writing bytecode')

def _require_loader(module, loader, skip):
    if isinstance(module, str):
        module = __import__(module)
    MODULE_KINDS = {BuiltinImporter: 'built-in', ExtensionFileLoader: 'extension', FrozenImporter: 'frozen', SourceFileLoader: 'pure Python'}
    expected = loader
    assert isinstance(expected, type), expected
    expected = MODULE_KINDS[expected]
    actual = module.__spec__.loader
    if not isinstance(actual, type):
        actual = type(actual)
    actual = MODULE_KINDS[actual]
    if actual != expected:
        err = f'expected module to be {expected}, got {module.__spec__}'
        if skip:
            raise unittest.SkipTest(err)
        raise Exception(err)
    return module

def require_builtin(module, *, skip=False):
    module = _require_loader(module, BuiltinImporter, skip)
    assert module.__spec__.origin == 'built-in', module.__spec__

def require_extension(module, *, skip=False):
    _require_loader(module, ExtensionFileLoader, skip)

def require_frozen(module, *, skip=True):
    module = _require_loader(module, FrozenImporter, skip)
    assert module.__spec__.origin == 'frozen', module.__spec__

def require_pure_python(module, *, skip=False):
    _require_loader(module, SourceFileLoader, skip)

def remove_files(name):
    for f in (name + '.py', name + '.pyc', name + '.pyw', name + '$py.class'):
        unlink(f)
    rmtree('__pycache__')

def no_rerun(reason):
    """Skip rerunning for a particular test.

    WARNING: Use this decorator with care; skipping rerunning makes it
    impossible to find reference leaks. Provide a clear reason for skipping the
    test using the 'reason' parameter.
    """

    def deco(func):
        _has_run = False

        def wrapper(self):
            nonlocal _has_run
            if _has_run:
                self.skipTest(reason)
            func(self)
            _has_run = True
        return wrapper
    return deco

if _testsinglephase is not None:

    def restore__testsinglephase(*, _orig=_testsinglephase):
        sys.modules.pop('_testsinglephase', None)
        _orig._clear_globals()
        _testinternalcapi.clear_extension('_testsinglephase', _orig.__file__)
        import _testsinglephase

def requires_singlephase_init(meth):
    """Decorator to skip if single-phase init modules are not supported."""
    if not isinstance(meth, type):

        def meth(self, _meth=meth):
            try:
                return _meth(self)
            finally:
                restore__testsinglephase()
    meth = cpython_only(meth)
    return unittest.skipIf(_testsinglephase is None, 'test requires _testsinglephase module')(meth)

def requires_subinterpreters(meth):
    """Decorator to skip a test if subinterpreters are not supported."""
    return unittest.skipIf(_interpreters is None, 'subinterpreters required')(meth)

class ModuleSnapshot(types.SimpleNamespace):
    """A representation of a module for testing.

    Fields:

    * id - the module's object ID
    * module - the actual module or an adequate substitute
       * __file__
       * __spec__
          * name
          * origin
    * ns - a copy (dict) of the module's __dict__ (or None)
    * ns_id - the object ID of the module's __dict__
    * cached - the sys.modules[mod.__spec__.name] entry (or None)
    * cached_id - the object ID of the sys.modules entry (or None)

    In cases where the value is not available (e.g. due to serialization),
    the value will be None.
    """
    _fields = tuple('id module ns ns_id cached cached_id'.split())

    @classmethod
    def from_module(cls, mod):
        name = mod.__spec__.name
        cached = sys.modules.get(name)
        return cls(id=id(mod), module=mod, ns=types.SimpleNamespace(**mod.__dict__), ns_id=id(mod.__dict__), cached=cached, cached_id=id(cached))
    SCRIPT = textwrap.dedent('\n        {imports}\n\n        name = {name!r}\n\n        {prescript}\n\n        mod = {name}\n\n        {body}\n\n        {postscript}\n        ')
    IMPORTS = textwrap.dedent('\n        import sys\n        ').strip()
    SCRIPT_BODY = textwrap.dedent('\n        # Capture the snapshot data.\n        cached = sys.modules.get(name)\n        snapshot = dict(\n            id=id(mod),\n            module=dict(\n                __file__=mod.__file__,\n                __spec__=dict(\n                    name=mod.__spec__.name,\n                    origin=mod.__spec__.origin,\n                ),\n            ),\n            ns=None,\n            ns_id=id(mod.__dict__),\n            cached=None,\n            cached_id=id(cached) if cached else None,\n        )\n        ').strip()
    CLEANUP_SCRIPT = textwrap.dedent('\n        # Clean up the module.\n        sys.modules.pop(name, None)\n        ').strip()

    @classmethod
    def build_script(cls, name, *, prescript=None, import_first=False, postscript=None, postcleanup=False):
        if postcleanup is True:
            postcleanup = cls.CLEANUP_SCRIPT
        elif isinstance(postcleanup, str):
            postcleanup = textwrap.dedent(postcleanup).strip()
            postcleanup = cls.CLEANUP_SCRIPT + os.linesep + postcleanup
        else:
            postcleanup = ''
        prescript = textwrap.dedent(prescript).strip() if prescript else ''
        postscript = textwrap.dedent(postscript).strip() if postscript else ''
        if postcleanup:
            if postscript:
                postscript = postscript + os.linesep * 2 + postcleanup
            else:
                postscript = postcleanup
        if import_first:
            prescript += textwrap.dedent(f'\n\n                # Now import the module.\n                assert name not in sys.modules\n                import {name}')
        return cls.SCRIPT.format(imports=cls.IMPORTS.strip(), name=name, prescript=prescript.strip(), body=cls.SCRIPT_BODY.strip(), postscript=postscript)

    @classmethod
    def parse(cls, text):
        raw = json.loads(text)
        mod = raw['module']
        mod['__spec__'] = types.SimpleNamespace(**mod['__spec__'])
        raw['module'] = types.SimpleNamespace(**mod)
        return cls(**raw)

    @classmethod
    def from_subinterp(cls, name, interpid=None, *, pipe=None, **script_kwds):
        if pipe is not None:
            return cls._from_subinterp(name, interpid, pipe, script_kwds)
        pipe = os.pipe()
        try:
            return cls._from_subinterp(name, interpid, pipe, script_kwds)
        finally:
            r, w = pipe
            os.close(r)
            os.close(w)

    @classmethod
    def _from_subinterp(cls, name, interpid, pipe, script_kwargs):
        r, w = pipe
        postscript = textwrap.dedent(f'\n            # Send the result over the pipe.\n            import json\n            import os\n            os.write({w}, json.dumps(snapshot).encode())\n\n            ')
        _postscript = script_kwargs.get('postscript')
        if _postscript:
            _postscript = textwrap.dedent(_postscript).lstrip()
            postscript += _postscript
        script_kwargs['postscript'] = postscript.strip()
        script = cls.build_script(name, **script_kwargs)
        if interpid is None:
            ret = run_in_subinterp(script)
            if ret != 0:
                raise AssertionError(f'{ret} != 0')
        else:
            _interpreters.run_string(interpid, script)
        text = os.read(r, 1000)
        return cls.parse(text.decode())


# --- test body ---
RUN_KWARGS = dict(allow_fork=False, allow_exec=False, allow_threads=True, allow_daemon_threads=False)
ISOLATED = dict(use_main_obmalloc=False, gil=2)
NOT_ISOLATED = {k: not v for k, v in ISOLATED.items()}

def check_compatible_fresh(name, *, strict=False, isolated=False):
    kwargs = dict(**RUN_KWARGS, **ISOLATED if isolated else NOT_ISOLATED, check_multi_interp_extensions=strict)
    _, out, err = script_helper.assert_python_ok('-c', textwrap.dedent(f"\n            import _testcapi, sys\n            assert (\n                {name!r} in sys.builtin_module_names or\n                {name!r} not in sys.modules\n            ), repr({name!r})\n            ret = _testcapi.run_in_subinterp_with_config(\n                {import_script(name, 'sys.stdout.fileno()')!r},\n                **{kwargs},\n            )\n            assert ret == 0, ret\n            "))

    assert err == b''

    assert out == b'okay'

def check_compatible_here(name, filename=None, *, strict=False, isolated=False):
    out = run_here(name, filename, check_singlephase_setting=strict, isolated=isolated)

    assert out == b'okay'

def check_incompatible_fresh(name, *, isolated=False):
    kwargs = dict(**RUN_KWARGS, **ISOLATED if isolated else NOT_ISOLATED, check_multi_interp_extensions=True)
    _, out, err = script_helper.assert_python_ok('-c', textwrap.dedent(f"\n            import _testcapi, sys\n            assert {name!r} not in sys.modules, {name!r}\n            ret = _testcapi.run_in_subinterp_with_config(\n                {import_script(name, 'sys.stdout.fileno()')!r},\n                **{kwargs},\n            )\n            assert ret == 0, ret\n            "))

    assert err == b''

    assert out.decode('utf-8') == f'ImportError: module {name} does not support loading in subinterpreters'

def check_incompatible_here(name, filename=None, *, isolated=False):
    out = run_here(name, filename, check_singlephase_setting=True, isolated=isolated)

    assert out.decode('utf-8') == f'ImportError: module {name} does not support loading in subinterpreters'

def import_script(name, fd, filename=None, check_override=None):
    override_text = ''
    if check_override is not None:
        override_text = f'\n                import _imp\n                _imp._override_multi_interp_extensions_check({check_override})\n                '
    if filename:
        return textwrap.dedent(f"\n                from importlib.util import spec_from_loader, module_from_spec\n                from importlib.machinery import ExtensionFileLoader\n                import os, sys\n                {override_text}\n                loader = ExtensionFileLoader({name!r}, {filename!r})\n                spec = spec_from_loader({name!r}, loader)\n                try:\n                    module = module_from_spec(spec)\n                    loader.exec_module(module)\n                except ImportError as exc:\n                    text = 'ImportError: ' + str(exc)\n                else:\n                    text = 'okay'\n                os.write({fd}, text.encode('utf-8'))\n                ")
    else:
        return textwrap.dedent(f"\n                import os, sys\n                {override_text}\n                try:\n                    import {name}\n                except ImportError as exc:\n                    text = 'ImportError: ' + str(exc)\n                else:\n                    text = 'okay'\n                os.write({fd}, text.encode('utf-8'))\n                ")

def pipe():
    r, w = os.pipe()
    pass
    pass
    if hasattr(os, 'set_blocking'):
        os.set_blocking(r, False)
    return (r, w)

def run_here(name, filename=None, *, check_singlephase_setting=False, check_singlephase_override=None, isolated=False):
    """
        Try importing the named module in a subinterpreter.

        The subinterpreter will be in the current process.
        The module will have already been imported in the main interpreter.
        Thus, for extension/builtin modules, the module definition will
        have been loaded already and cached globally.

        "check_singlephase_setting" determines whether or not
        the interpreter will be configured to check for modules
        that are not compatible with use in multiple interpreters.

        This should always return "okay" for all modules if the
        setting is False (with no override).
        """
    __import__(name)
    kwargs = dict(**RUN_KWARGS, **ISOLATED if isolated else NOT_ISOLATED, check_multi_interp_extensions=check_singlephase_setting)
    r, w = pipe()
    script = import_script(name, w, filename, check_singlephase_override)
    ret = run_in_subinterp_with_config(script, **kwargs)

    assert ret == 0
    return os.read(r, 100)
module = '_testsinglephase'
require_extension(module)
check_compatible_here(module, strict=False)
check_incompatible_here(module)
check_incompatible_fresh(module)
print("SubinterpImportTests::test_single_init_extension_compat: ok")
