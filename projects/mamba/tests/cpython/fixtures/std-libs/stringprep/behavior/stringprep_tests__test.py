# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "stringprep"
# dimension = "behavior"
# case = "stringprep_tests__test"
# subject = "cpython.test_stringprep.StringprepTests.test"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_stringprep.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_stringprep.py::StringprepTests::test
"""Auto-ported test: StringprepTests::test (CPython 3.12 oracle)."""


import unittest
from stringprep import *


# --- test body ---

assert in_table_a1('ȡ')

assert not in_table_a1('Ȣ')

assert in_table_b1('\xad')

assert not in_table_b1('®')

assert map_table_b2('A')

assert map_table_b2('a')

assert map_table_b3('A')

assert map_table_b3('a')

assert in_table_c11(' ')

assert not in_table_c11('!')

assert in_table_c12('\xa0')

assert not in_table_c12('¡')

assert in_table_c12('\xa0')

assert not in_table_c12('¡')

assert in_table_c11_c12('\xa0')

assert not in_table_c11_c12('¡')

assert in_table_c21('\x1f')

assert not in_table_c21(' ')

assert in_table_c22('\x9f')

assert not in_table_c22('\xa0')

assert in_table_c21_c22('\x9f')

assert not in_table_c21_c22('\xa0')

assert in_table_c3('\ue000')

assert not in_table_c3('豈')

assert in_table_c4('\uffff')

assert not in_table_c4('\x00')

assert in_table_c5('\ud800')

assert not in_table_c5('\ud7ff')

assert in_table_c6('\ufff9')

assert not in_table_c6('\ufffe')

assert in_table_c7('⿰')

assert not in_table_c7('\u2ffc')

assert in_table_c8('̀')

assert not in_table_c8('͂')

assert in_table_d1('־')

assert not in_table_d1('ֿ')

assert in_table_d2('A')

assert not in_table_d2('@')
print("StringprepTests::test: ok")
