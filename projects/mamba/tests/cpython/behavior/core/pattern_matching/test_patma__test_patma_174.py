# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_patma__test_patma_174"
# subject = "cpython.test_patma.TestPatma.test_patma_174"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestPatma::test_patma_174
"""Auto-ported test: TestPatma::test_patma_174 (CPython 3.12 oracle)."""


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
        case 401:
            return 'Unauthorized'
        case 403:
            return 'Forbidden'
        case 404:
            return 'Not found'
        case 418:
            return "I'm a teapot"
        case _:
            return 'Something else'

assert http_error(400) == 'Bad request'

assert http_error(401) == 'Unauthorized'

assert http_error(403) == 'Forbidden'

assert http_error(404) == 'Not found'

assert http_error(418) == "I'm a teapot"

assert http_error(123) == 'Something else'

assert http_error('400') == 'Something else'

assert http_error(401 | 403 | 404) == 'Something else'
print("TestPatma::test_patma_174: ok")
