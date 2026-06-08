# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "interpreters"
# dimension = "behavior"
# case = "test_send_recv__test_send_channel_does_not_exist"
# subject = "cpython.test_interpreters.TestSendRecv.test_send_channel_does_not_exist"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_interpreters.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_interpreters.py::TestSendRecv::test_send_channel_does_not_exist
"""Auto-ported test: TestSendRecv::test_send_channel_does_not_exist (CPython 3.12 oracle)."""


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
ch = interpreters.SendChannel(1000000)
try:
    ch.send(b'spam')
    raise AssertionError('expected interpreters.ChannelNotFoundError')
except interpreters.ChannelNotFoundError:
    pass
print("TestSendRecv::test_send_channel_does_not_exist: ok")
