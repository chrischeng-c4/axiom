# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pwd"
# dimension = "behavior"
# case = "pwd_test__test_values"
# subject = "cpython.test_pwd.PwdTest.test_values"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pwd.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_pwd.py::PwdTest::test_values
"""Auto-ported test: PwdTest::test_values (CPython 3.12 oracle)."""


import sys
import unittest
from test.support import import_helper


pwd = import_helper.import_module('pwd')


# --- test body ---
entries = pwd.getpwall()
for e in entries:

    assert len(e) == 7

    assert e[0] == e.pw_name

    assert isinstance(e.pw_name, str)

    assert e[1] == e.pw_passwd

    assert isinstance(e.pw_passwd, str)

    assert e[2] == e.pw_uid

    assert isinstance(e.pw_uid, int)

    assert e[3] == e.pw_gid

    assert isinstance(e.pw_gid, int)

    assert e[4] == e.pw_gecos

    assert type(e.pw_gecos) in (str, type(None))

    assert e[5] == e.pw_dir

    assert isinstance(e.pw_dir, str)

    assert e[6] == e.pw_shell

    assert isinstance(e.pw_shell, str)
print("PwdTest::test_values: ok")
