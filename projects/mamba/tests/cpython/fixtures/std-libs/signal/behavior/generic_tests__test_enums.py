# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "generic_tests__test_enums"
# subject = "cpython.test_signal.GenericTests.test_enums"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_signal.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_signal.py::GenericTests::test_enums
"""Auto-ported test: GenericTests::test_enums (CPython 3.12 oracle)."""


import enum
import errno
import functools
import inspect
import os
import random
import signal
import socket
import statistics
import subprocess
import sys
import threading
import time
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, spawn_python
from test.support import threading_helper


try:
    import _testcapi
except ImportError:
    _testcapi = None

def tearDownModule():
    support.reap_children()


# --- test body ---
for name in dir(signal):
    sig = getattr(signal, name)
    if name in {'SIG_DFL', 'SIG_IGN'}:

        assert isinstance(sig, signal.Handlers)
    elif name in {'SIG_BLOCK', 'SIG_UNBLOCK', 'SIG_SETMASK'}:

        assert isinstance(sig, signal.Sigmasks)
    elif name.startswith('SIG') and (not name.startswith('SIG_')):

        assert isinstance(sig, signal.Signals)
    elif name.startswith('CTRL_'):

        assert isinstance(sig, signal.Signals)

        assert sys.platform == 'win32'
CheckedSignals = enum._old_convert_(enum.IntEnum, 'Signals', 'signal', lambda name: name.isupper() and (name.startswith('SIG') and (not name.startswith('SIG_'))) or name.startswith('CTRL_'), source=signal)
enum._test_simple_enum(CheckedSignals, signal.Signals)
CheckedHandlers = enum._old_convert_(enum.IntEnum, 'Handlers', 'signal', lambda name: name in ('SIG_DFL', 'SIG_IGN'), source=signal)
enum._test_simple_enum(CheckedHandlers, signal.Handlers)
Sigmasks = getattr(signal, 'Sigmasks', None)
if Sigmasks is not None:
    CheckedSigmasks = enum._old_convert_(enum.IntEnum, 'Sigmasks', 'signal', lambda name: name in ('SIG_BLOCK', 'SIG_UNBLOCK', 'SIG_SETMASK'), source=signal)
    enum._test_simple_enum(CheckedSigmasks, Sigmasks)
print("GenericTests::test_enums: ok")
