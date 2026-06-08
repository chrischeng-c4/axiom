# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_175"
# subject = "cpython.test_patma.TestPatma.test_patma_175"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_175
"""Auto-ported test: TestPatma::test_patma_175 (CPython 3.12 oracle)."""


import array
import collections
import dataclasses
import dis
import enum
import inspect
import sys
import unittest


@dataclasses.dataclass
class Point:
    x: int
    y: int


# --- test body ---
def http_error(status):
    match status:
        case 400:
            return 'Bad request'
        case 401 | 403 | 404:
            return 'Not allowed'
        case 418:
            return "I'm a teapot"

assert http_error(400) == 'Bad request'

assert http_error(401) == 'Not allowed'

assert http_error(403) == 'Not allowed'

assert http_error(404) == 'Not allowed'

assert http_error(418) == "I'm a teapot"

assert http_error(123) is None

assert http_error('400') is None

assert http_error(401 | 403 | 404) is None
print("TestPatma::test_patma_175: ok")
