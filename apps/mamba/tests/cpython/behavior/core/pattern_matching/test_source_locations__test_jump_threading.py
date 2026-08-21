# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "pattern_matching"
# dimension = "behavior"
# case = "test_source_locations__test_jump_threading"
# subject = "cpython.test_patma.TestSourceLocations.test_jump_threading"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_patma.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_patma.py::TestSourceLocations::test_jump_threading
"""Auto-ported test: TestSourceLocations::test_jump_threading (CPython 3.12 oracle)."""


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
def f():
    x = 0
    v = 1
    match v:
        case 1:
            if x < 0:
                x = 1
        case 2:
            if x < 0:
                x = 1
    x += 1
for inst in dis.get_instructions(f):
    if inst.opcode in dis.hasjrel or inst.opcode in dis.hasjabs:

        assert inst.positions.lineno is not None
print("TestSourceLocations::test_jump_threading: ok")
