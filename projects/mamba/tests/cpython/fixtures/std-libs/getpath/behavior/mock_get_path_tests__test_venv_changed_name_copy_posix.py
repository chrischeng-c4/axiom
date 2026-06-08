# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "getpath"
# dimension = "behavior"
# case = "mock_get_path_tests__test_venv_changed_name_copy_posix"
# subject = "cpython.test_getpath.MockGetPathTests.test_venv_changed_name_copy_posix"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_getpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_getpath.py::MockGetPathTests::test_venv_changed_name_copy_posix
"""Auto-ported test: MockGetPathTests::test_venv_changed_name_copy_posix (CPython 3.12 oracle)."""


import copy
import ntpath
import pathlib
import posixpath
import unittest
from test.support import verbose


try:
    SOURCE = (pathlib.Path(__file__).absolute().parent.parent.parent / 'Modules/getpath.py').read_bytes()
except FileNotFoundError:
    from _testinternalcapi import get_getpath_codeobject
    SOURCE = get_getpath_codeobject()

DEFAULT_NAMESPACE = dict(PREFIX='', EXEC_PREFIX='', PYTHONPATH='', VPATH='', PLATLIBDIR='', PYDEBUGEXT='', VERSION_MAJOR=9, VERSION_MINOR=8, PYWINVER=None, EXE_SUFFIX=None, ENV_PATH='', ENV_PYTHONHOME='', ENV_PYTHONEXECUTABLE='', ENV___PYVENV_LAUNCHER__='', argv0='', py_setpath='', real_executable='', executable_dir='', library='', winreg=None, build_prefix=None, venv_prefix=None)

DEFAULT_CONFIG = dict(home=None, platlibdir=None, pythonpath=None, program_name=None, prefix=None, exec_prefix=None, base_prefix=None, base_exec_prefix=None, executable=None, base_executable='', stdlib_dir=None, platstdlib_dir=None, module_search_paths=None, module_search_paths_set=0, pythonpath_env=None, argv=None, orig_argv=None, isolated=0, use_environment=1, use_site=1)

class MockNTNamespace(dict):

    def __init__(self, *a, argv0=None, config=None, **kw):
        self.update(DEFAULT_NAMESPACE)
        self['config'] = DEFAULT_CONFIG.copy()
        self['os_name'] = 'nt'
        self['PLATLIBDIR'] = 'DLLs'
        self['PYWINVER'] = '9.8-XY'
        self['VPATH'] = '..\\..'
        super().__init__(*a, **kw)
        if argv0:
            self['config']['orig_argv'] = [argv0]
        if config:
            self['config'].update(config)
        self._files = {}
        self._links = {}
        self._dirs = set()
        self._warnings = []

    def add_known_file(self, path, lines=None):
        self._files[path.casefold()] = list(lines or ())
        self.add_known_dir(path.rpartition('\\')[0])

    def add_known_xfile(self, path):
        self.add_known_file(path)

    def add_known_link(self, path, target):
        self._links[path.casefold()] = target

    def add_known_dir(self, path):
        p = path.rstrip('\\').casefold()
        while p:
            self._dirs.add(p)
            p = p.rpartition('\\')[0]

    def __missing__(self, key):
        try:
            return getattr(self, key)
        except AttributeError:
            raise KeyError(key) from None

    def abspath(self, path):
        if self.isabs(path):
            return path
        return self.joinpath('C:\\Absolute', path)

    def basename(self, path):
        return path.rpartition('\\')[2]

    def dirname(self, path):
        name = path.rstrip('\\').rpartition('\\')[0]
        if name[1:] == ':':
            return name + '\\'
        return name

    def hassuffix(self, path, suffix):
        return path.casefold().endswith(suffix.casefold())

    def isabs(self, path):
        return path[1:3] == ':\\'

    def isdir(self, path):
        if verbose:
            print('Check if', path, 'is a dir')
        return path.casefold() in self._dirs

    def isfile(self, path):
        if verbose:
            print('Check if', path, 'is a file')
        return path.casefold() in self._files

    def ismodule(self, path):
        if verbose:
            print('Check if', path, 'is a module')
        path = path.casefold()
        return path in self._files and path.rpartition('.')[2] == 'py'.casefold()

    def isxfile(self, path):
        if verbose:
            print('Check if', path, 'is a executable')
        path = path.casefold()
        return path in self._files and path.rpartition('.')[2] == 'exe'.casefold()

    def joinpath(self, *path):
        return ntpath.normpath(ntpath.join(*path))

    def readlines(self, path):
        try:
            return self._files[path.casefold()]
        except KeyError:
            raise FileNotFoundError(path) from None

    def realpath(self, path, _trail=None):
        if verbose:
            print('Read link from', path)
        try:
            link = self._links[path.casefold()]
        except KeyError:
            return path
        if _trail is None:
            _trail = set()
        elif link.casefold() in _trail:
            raise OSError('circular link')
        _trail.add(link.casefold())
        return self.realpath(link, _trail)

    def warn(self, message):
        self._warnings.append(message)
        if verbose:
            print(message)

class MockWinreg:
    HKEY_LOCAL_MACHINE = 'HKLM'
    HKEY_CURRENT_USER = 'HKCU'

    def __init__(self, keys):
        self.keys = {k.casefold(): v for k, v in keys.items()}
        self.open = {}

    def __repr__(self):
        return '<MockWinreg>'

    def __eq__(self, other):
        return isinstance(other, type(self))

    def open_keys(self):
        return list(self.open)

    def OpenKeyEx(self, hkey, subkey):
        if verbose:
            print(f'OpenKeyEx({hkey}, {subkey})')
        key = f'{hkey}\\{subkey}'.casefold()
        if key in self.keys:
            self.open[key] = self.open.get(key, 0) + 1
            return key
        raise FileNotFoundError()

    def CloseKey(self, hkey):
        if verbose:
            print(f'CloseKey({hkey})')
        hkey = hkey.casefold()
        if hkey not in self.open:
            raise RuntimeError('key is not open')
        self.open[hkey] -= 1
        if not self.open[hkey]:
            del self.open[hkey]

    def EnumKey(self, hkey, i):
        if verbose:
            print(f'EnumKey({hkey}, {i})')
        hkey = hkey.casefold()
        if hkey not in self.open:
            raise RuntimeError('key is not open')
        prefix = f'{hkey}\\'
        subkeys = [k[len(prefix):] for k in sorted(self.keys) if k.startswith(prefix)]
        subkeys[:] = [k for k in subkeys if '\\' not in k]
        for j, n in enumerate(subkeys):
            if j == i:
                return n.removeprefix(prefix)
        raise OSError('end of enumeration')

    def QueryValue(self, hkey, subkey):
        if verbose:
            print(f'QueryValue({hkey}, {subkey})')
        hkey = hkey.casefold()
        if hkey not in self.open:
            raise RuntimeError('key is not open')
        if subkey:
            subkey = subkey.casefold()
            hkey = f'{hkey}\\{subkey}'
        try:
            return self.keys[hkey]
        except KeyError:
            raise OSError()

class MockPosixNamespace(dict):

    def __init__(self, *a, argv0=None, config=None, **kw):
        self.update(DEFAULT_NAMESPACE)
        self['config'] = DEFAULT_CONFIG.copy()
        self['os_name'] = 'posix'
        self['PLATLIBDIR'] = 'lib'
        self['WITH_NEXT_FRAMEWORK'] = 0
        super().__init__(*a, **kw)
        if argv0:
            self['config']['orig_argv'] = [argv0]
        if config:
            self['config'].update(config)
        self._files = {}
        self._xfiles = set()
        self._links = {}
        self._dirs = set()
        self._warnings = []

    def add_known_file(self, path, lines=None):
        self._files[path] = list(lines or ())
        self.add_known_dir(path.rpartition('/')[0])

    def add_known_xfile(self, path):
        self.add_known_file(path)
        self._xfiles.add(path)

    def add_known_link(self, path, target):
        self._links[path] = target

    def add_known_dir(self, path):
        p = path.rstrip('/')
        while p:
            self._dirs.add(p)
            p = p.rpartition('/')[0]

    def __missing__(self, key):
        try:
            return getattr(self, key)
        except AttributeError:
            raise KeyError(key) from None

    def abspath(self, path):
        if self.isabs(path):
            return path
        return self.joinpath('/Absolute', path)

    def basename(self, path):
        return path.rpartition('/')[2]

    def dirname(self, path):
        return path.rstrip('/').rpartition('/')[0]

    def hassuffix(self, path, suffix):
        return path.endswith(suffix)

    def isabs(self, path):
        return path[0:1] == '/'

    def isdir(self, path):
        if verbose:
            print('Check if', path, 'is a dir')
        return path in self._dirs

    def isfile(self, path):
        if verbose:
            print('Check if', path, 'is a file')
        return path in self._files

    def ismodule(self, path):
        if verbose:
            print('Check if', path, 'is a module')
        return path in self._files and path.rpartition('.')[2] == 'py'

    def isxfile(self, path):
        if verbose:
            print('Check if', path, 'is an xfile')
        return path in self._xfiles

    def joinpath(self, *path):
        return posixpath.normpath(posixpath.join(*path))

    def readlines(self, path):
        try:
            return self._files[path]
        except KeyError:
            raise FileNotFoundError(path) from None

    def realpath(self, path, _trail=None):
        if verbose:
            print('Read link from', path)
        try:
            link = self._links[path]
        except KeyError:
            return path
        if _trail is None:
            _trail = set()
        elif link in _trail:
            raise OSError('circular link')
        _trail.add(link)
        return self.realpath(link, _trail)

    def warn(self, message):
        self._warnings.append(message)
        if verbose:
            print(message)

def diff_dict(before, after, prefix='global'):
    diff = []
    for k in sorted(before):
        if k[:2] == '__':
            continue
        if k == 'config':
            diff_dict(before[k], after[k], prefix='config')
            continue
        if k in after and after[k] != before[k]:
            diff.append((k, before[k], after[k]))
    if not diff:
        return
    max_k = max((len(k) for k, _, _ in diff))
    indent = ' ' * (len(prefix) + 1 + max_k)
    if verbose:
        for k, b, a in diff:
            if b:
                print('{}.{} -{!r}\n{} +{!r}'.format(prefix, k.ljust(max_k), b, indent, a))
            else:
                print('{}.{} +{!r}'.format(prefix, k.ljust(max_k), a))

def dump_dict(before, after, prefix='global'):
    if not verbose or not after:
        return
    max_k = max((len(k) for k in after))
    for k, v in sorted(after.items(), key=lambda i: i[0]):
        if k[:2] == '__':
            continue
        if k == 'config':
            dump_dict(before[k], after[k], prefix='config')
            continue
        try:
            if v != before[k]:
                print('{}.{} {!r} (was {!r})'.format(prefix, k.ljust(max_k), v, before[k]))
                continue
        except KeyError:
            pass
        print('{}.{} {!r}'.format(prefix, k.ljust(max_k), v))

def getpath(ns, keys):
    before = copy.deepcopy(ns)
    failed = True
    try:
        exec(SOURCE, ns)
        failed = False
    finally:
        if failed:
            dump_dict(before, ns)
        else:
            diff_dict(before, ns)
    return {k: ns['config'].get(k, ns.get(k, ...)) for k in keys}


# --- test body ---
"""Test a venv --copies layout on *nix that lacks a distributed 'python'"""
ns = MockPosixNamespace(argv0='python', PREFIX='/usr', ENV_PATH='/venv/bin:/usr/bin')
ns.add_known_xfile('/usr/bin/python9')
ns.add_known_xfile('/venv/bin/python')
ns.add_known_file('/usr/lib/python9.8/os.py')
ns.add_known_dir('/usr/lib/python9.8/lib-dynload')
ns.add_known_file('/venv/pyvenv.cfg', ['home = /usr/bin'])
expected = dict(executable='/venv/bin/python', prefix='/usr', exec_prefix='/usr', base_executable='/usr/bin/python9', base_prefix='/usr', base_exec_prefix='/usr', module_search_paths_set=1, module_search_paths=['/usr/lib/python98.zip', '/usr/lib/python9.8', '/usr/lib/python9.8/lib-dynload'])
actual = getpath(ns, expected)

assert expected == actual
print("MockGetPathTests::test_venv_changed_name_copy_posix: ok")
