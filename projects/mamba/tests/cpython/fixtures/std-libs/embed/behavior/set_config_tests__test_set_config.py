# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "embed"
# dimension = "behavior"
# case = "set_config_tests__test_set_config"
# subject = "cpython.test_embed.SetConfigTests.test_set_config"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_embed.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_embed.py::SetConfigTests::test_set_config
"""Auto-ported test: SetConfigTests::test_set_config (CPython 3.12 oracle)."""


from test import support
from test.support import import_helper, os_helper, MS_WINDOWS
import unittest
from collections import namedtuple
import contextlib
import io
import json
import os
import os.path
import re
import shutil
import subprocess
import sys
import sysconfig
import tempfile
import textwrap


if not support.has_subprocess_support:
    raise unittest.SkipTest('test module requires subprocess')

MACOS = sys.platform == 'darwin'

PYMEM_ALLOCATOR_NOT_SET = 0

PYMEM_ALLOCATOR_DEBUG = 2

PYMEM_ALLOCATOR_MALLOC = 3

API_COMPAT = 1

API_PYTHON = 2

API_ISOLATED = 3

INIT_LOOPS = 4

MAX_HASH_SEED = 4294967295

STDLIB_INSTALL = os.path.join(sys.prefix, sys.platlibdir, f'python{sys.version_info.major}.{sys.version_info.minor}')

if not os.path.isfile(os.path.join(STDLIB_INSTALL, 'os.py')):
    STDLIB_INSTALL = None

def debug_build(program):
    program = os.path.basename(program)
    name = os.path.splitext(program)[0]
    return name.casefold().endswith('_d'.casefold())

def remove_python_envvars():
    env = dict(os.environ)
    for key in list(env):
        if key.startswith('PYTHON'):
            del env[key]
    return env

class EmbeddingTestsMixin:

    def setUp(self):
        exename = '_testembed'
        builddir = os.path.dirname(sys.executable)
        if MS_WINDOWS:
            ext = ('_d' if debug_build(sys.executable) else '') + '.exe'
            exename += ext
            exepath = builddir
        else:
            exepath = os.path.join(builddir, 'Programs')
        self.test_exe = exe = os.path.join(exepath, exename)
        if not os.path.exists(exe):
            self.skipTest("%r doesn't exist" % exe)
        self.oldcwd = os.getcwd()
        os.chdir(builddir)

    def tearDown(self):
        os.chdir(self.oldcwd)

    def run_embedded_interpreter(self, *args, env=None, timeout=None, returncode=0, input=None, cwd=None):
        """Runs a test in the embedded interpreter"""
        cmd = [self.test_exe]
        cmd.extend(args)
        if env is not None and MS_WINDOWS:
            env = env.copy()
            env['SYSTEMROOT'] = os.environ['SYSTEMROOT']
        p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, universal_newlines=True, env=env, cwd=cwd)
        try:
            out, err = p.communicate(input=input, timeout=timeout)
        except:
            p.terminate()
            p.wait()
            raise
        if p.returncode != returncode and support.verbose:
            print(f'--- {cmd} failed ---')
            print(f'stdout:\n{out}')
            print(f'stderr:\n{err}')
            print('------')
        self.assertEqual(p.returncode, returncode, 'bad returncode %d, stderr is %r' % (p.returncode, err))
        return (out, err)

    def run_repeated_init_and_subinterpreters(self):
        out, err = self.run_embedded_interpreter('test_repeated_init_and_subinterpreters')
        self.assertEqual(err, '')
        interp_pat = '^interp (\\d+) <(0x[\\dA-F]+)>, thread state <(0x[\\dA-F]+)>: id\\(modules\\) = ([\\d]+)$'
        Interp = namedtuple('Interp', 'id interp tstate modules')
        numloops = 1
        current_run = []
        for line in out.splitlines():
            if line == '--- Pass {} ---'.format(numloops):
                self.assertEqual(len(current_run), 0)
                if support.verbose > 1:
                    print(line)
                numloops += 1
                continue
            self.assertLess(len(current_run), 5)
            match = re.match(interp_pat, line)
            if match is None:
                self.assertRegex(line, interp_pat)
            interp = Interp(*match.groups())
            if support.verbose > 1:
                print(interp)
            self.assertTrue(interp.interp)
            self.assertTrue(interp.tstate)
            self.assertTrue(interp.modules)
            current_run.append(interp)
            if len(current_run) == 5:
                main = current_run[0]
                self.assertEqual(interp, main)
                yield current_run
                current_run = []


# --- test body ---
import_helper.import_module('_testcapi')
cmd = [sys.executable, '-X', 'utf8', '-I', '-m', 'test._test_embed_set_config']
proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, encoding='utf-8', errors='backslashreplace')
if proc.returncode and support.verbose:
    print(proc.stdout)
    print(proc.stderr)

assert proc.returncode == 0
print("SetConfigTests::test_set_config: ok")
