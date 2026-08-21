# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_xxsubinterpreters"
# dimension = "behavior"
# case = "destroy_tests__test_from_other_thread"
# subject = "cpython.test__xxsubinterpreters.DestroyTests.test_from_other_thread"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test__xxsubinterpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test__xxsubinterpreters.py::DestroyTests::test_from_other_thread
"""Auto-ported test: DestroyTests::test_from_other_thread (CPython 3.12 oracle)."""


import contextlib
import itertools
import os
import pickle
import sys
from textwrap import dedent
import threading
import unittest
from test import support
from test.support import import_helper
from test.support import script_helper


interpreters = import_helper.import_module('_xxsubinterpreters')

_testcapi = import_helper.import_module('_testcapi')

def _captured_script(script):
    r, w = os.pipe()
    indented = script.replace('\n', '\n                ')
    wrapped = dedent(f"""\n        import contextlib\n        with open({w}, 'w', encoding="utf-8") as spipe:\n            with contextlib.redirect_stdout(spipe):\n                {indented}\n        """)
    return (wrapped, open(r, encoding='utf-8'))

def _run_output(interp, request, shared=None):
    script, rpipe = _captured_script(request)
    with rpipe:
        interpreters.run_string(interp, script, shared)
        return rpipe.read()

def _wait_for_interp_to_run(interp, timeout=None):
    if timeout is None:
        timeout = support.SHORT_TIMEOUT
    for _ in support.sleeping_retry(timeout, error=False):
        if interpreters.is_running(interp):
            break
    else:
        raise RuntimeError('interp is not running')

@contextlib.contextmanager
def _running(interp):
    r, w = os.pipe()

    def run():
        interpreters.run_string(interp, dedent(f'\n            # wait for "signal"\n            with open({r}, encoding="utf-8") as rpipe:\n                rpipe.read()\n            '))
    t = threading.Thread(target=run)
    t.start()
    _wait_for_interp_to_run(interp)
    yield
    with open(w, 'w', encoding='utf-8') as spipe:
        spipe.write('done')
    t.join()

def clean_up_interpreters():
    for id in interpreters.list_all():
        if id == 0:
            continue
        try:
            interpreters.destroy(id)
        except RuntimeError:
            pass


# --- test body ---
id = interpreters.create()

def f():
    interpreters.destroy(id)
t = threading.Thread(target=f)
t.start()
t.join()
print("DestroyTests::test_from_other_thread: ok")
