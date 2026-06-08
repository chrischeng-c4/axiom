# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "grp"
# dimension = "behavior"
# case = "group_database_test_case__test_values"
# subject = "cpython.test_grp.GroupDatabaseTestCase.test_values"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_grp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_grp.py::GroupDatabaseTestCase::test_values
"""Auto-ported test: GroupDatabaseTestCase::test_values (CPython 3.12 oracle)."""


import unittest
from test.support import import_helper


'Test script for the grp module.'

grp = import_helper.import_module('grp')


# --- test body ---
def check_value(value):

    assert len(value) == 4

    assert value[0] == value.gr_name

    assert isinstance(value.gr_name, str)

    assert value[1] == value.gr_passwd

    assert isinstance(value.gr_passwd, str)

    assert value[2] == value.gr_gid

    assert isinstance(value.gr_gid, int)

    assert value[3] == value.gr_mem

    assert isinstance(value.gr_mem, list)
entries = grp.getgrall()
for e in entries:
    check_value(e)
print("GroupDatabaseTestCase::test_values: ok")
