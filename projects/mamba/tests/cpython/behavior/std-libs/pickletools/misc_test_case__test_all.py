# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickletools"
# dimension = "behavior"
# case = "misc_test_case__test_all"
# subject = "cpython.test_pickletools.MiscTestCase.test__all__"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pickletools.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pickletools.py::MiscTestCase::test__all__
"""Auto-ported test: MiscTestCase::test__all__ (CPython 3.12 oracle)."""


import pickletools
import unittest
from test import support


not_exported = {
    "bytes_types",
    "UP_TO_NEWLINE", "TAKEN_FROM_ARGUMENT1",
    "TAKEN_FROM_ARGUMENT4", "TAKEN_FROM_ARGUMENT4U",
    "TAKEN_FROM_ARGUMENT8U", "ArgumentDescriptor",
    "read_uint1", "read_uint2", "read_int4", "read_uint4",
    "read_uint8", "read_stringnl", "read_stringnl_noescape",
    "read_stringnl_noescape_pair", "read_string1",
    "read_string4", "read_bytes1", "read_bytes4",
    "read_bytes8", "read_bytearray8", "read_unicodestringnl",
    "read_unicodestring1", "read_unicodestring4",
    "read_unicodestring8", "read_decimalnl_short",
    "read_decimalnl_long", "read_floatnl", "read_float8",
    "read_long1", "read_long4",
    "uint1", "uint2", "int4", "uint4", "uint8", "stringnl",
    "stringnl_noescape", "stringnl_noescape_pair", "string1",
    "string4", "bytes1", "bytes4", "bytes8", "bytearray8",
    "unicodestringnl", "unicodestring1", "unicodestring4",
    "unicodestring8", "decimalnl_short", "decimalnl_long",
    "floatnl", "float8", "long1", "long4",
    "StackObject",
    "pyint", "pylong", "pyinteger_or_bool", "pybool", "pyfloat",
    "pybytes_or_str", "pystring", "pybytes", "pybytearray",
    "pyunicode", "pynone", "pytuple", "pylist", "pydict",
    "pyset", "pyfrozenset", "pybuffer", "anyobject",
    "markobject", "stackslice", "OpcodeInfo", "opcodes",
    "code2op",
}
support.check__all__(unittest.TestCase(), pickletools, not_exported=not_exported)

print("MiscTestCase::test__all__: ok")
