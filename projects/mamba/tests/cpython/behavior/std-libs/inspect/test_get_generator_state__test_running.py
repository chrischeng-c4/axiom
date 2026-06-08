# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_get_generator_state__test_running"
# subject = "cpython.test_inspect.TestGetGeneratorState.test_running"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import asyncio
import builtins
import collections
import datetime
import functools
import gc
import importlib
import inspect
import io
import linecache
import os
import dis
from os.path import normcase
import _pickle
import pickle
import shutil
import sys
import types
import textwrap
from typing import Unpack
import unicodedata
import warnings
import weakref

def _generatorstate():
    return inspect.getgeneratorstate(self_generator)

def number_generator():
    for number in range(5):
        yield number
self_generator = number_generator()

def running_check_generator():
    for number in range(5):
        assert _generatorstate() == inspect.GEN_RUNNING
        yield number
        assert _generatorstate() == inspect.GEN_RUNNING
self_generator = running_check_generator()
next(self_generator)
next(self_generator)

print("TestGetGeneratorState::test_running: ok")
