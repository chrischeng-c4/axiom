# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "interpreters"
# dimension = "behavior"
# case = "test_interpreter_close__test_subthreads_still_running"
# subject = "cpython.test_interpreters.TestInterpreterClose.test_subthreads_still_running"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_interpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_interpreters.py::TestInterpreterClose::test_subthreads_still_running
"""Auto-ported test: TestInterpreterClose::test_subthreads_still_running (CPython 3.12 oracle)."""


import contextlib
import json
import os
import os.path
import sys
import threading
from textwrap import dedent
import unittest
import time
from test import support
from test.support import import_helper
from test.support import threading_helper
from test.support import os_helper
from test.support import interpreters


_interpreters = import_helper.import_module('_xxsubinterpreters')

_channels = import_helper.import_module('_xxinterpchannels')

def _captured_script(script):
    r, w = os.pipe()
    indented = script.replace('\n', '\n                ')
    wrapped = dedent(f"\n        import contextlib\n        with open({w}, 'w', encoding='utf-8') as spipe:\n            with contextlib.redirect_stdout(spipe):\n                {indented}\n        ")
    return (wrapped, open(r, encoding='utf-8'))

def clean_up_interpreters():
    for interp in interpreters.list_all():
        if interp.id == 0:
            continue
        try:
            interp.close()
        except RuntimeError:
            pass

def _run_output(interp, request, channels=None):
    script, rpipe = _captured_script(request)
    with rpipe:
        interp.run(script, channels=channels)
        return rpipe.read()

@contextlib.contextmanager
def _running(interp):
    r, w = os.pipe()

    def run():
        interp.run(dedent(f'\n            # wait for "signal"\n            with open({r}) as rpipe:\n                rpipe.read()\n            '))
    t = threading.Thread(target=run)
    t.start()
    yield
    with open(w, 'w') as spipe:
        spipe.write('done')
    t.join()


# --- test body ---
def os_pipe():
    r, w = os.pipe()

    def cleanup():
        try:
            os.close(w)
        except Exception:
            pass
        try:
            os.close(r)
        except Exception:
            pass
    pass
    return (r, w)
r_interp, w_interp = os_pipe()
r_thread, w_thread = os_pipe()
FINISHED = b'F'
interp = interpreters.create()
interp.run(f'if True:\n            import os\n            import threading\n            import time\n\n            done = False\n\n            def notify_fini():\n                global done\n                done = True\n                t.join()\n            threading._register_atexit(notify_fini)\n\n            def task():\n                while not done:\n                    time.sleep(0.1)\n                os.write({w_interp}, {FINISHED!r})\n            t = threading.Thread(target=task)\n            t.start()\n            ')
interp.close()

assert os.read(r_interp, 1) == FINISHED
print("TestInterpreterClose::test_subthreads_still_running: ok")
