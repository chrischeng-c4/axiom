# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "select"
# dimension = "behavior"
# case = "select_test_case__test_returned_list_identity"
# subject = "cpython.test_select.SelectTestCase.test_returned_list_identity"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_select.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_select.py::SelectTestCase::test_returned_list_identity
"""Auto-ported test: SelectTestCase::test_returned_list_identity (CPython 3.12 oracle)."""


import errno
import select
import subprocess
import sys
import textwrap
import unittest
from test import support


support.requires_working_socket(module=True)

def tearDownModule():
    support.reap_children()


# --- test body ---
r, w, x = select.select([], [], [], 1)

assert r is not w

assert r is not x

assert w is not x
print("SelectTestCase::test_returned_list_identity: ok")
