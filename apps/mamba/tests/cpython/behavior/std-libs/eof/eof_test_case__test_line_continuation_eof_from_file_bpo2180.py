# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "eof"
# dimension = "behavior"
# case = "eof_test_case__test_line_continuation_eof_from_file_bpo2180"
# subject = "cpython.test_eof.EOFTestCase.test_line_continuation_EOF_from_file_bpo2180"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_eof.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_eof.py::EOFTestCase::test_line_continuation_EOF_from_file_bpo2180
"""Auto-ported test: EOFTestCase::test_line_continuation_EOF_from_file_bpo2180."""


import sys
from codecs import BOM_UTF8
from test.support import os_helper
from test.support import script_helper


if not sys.executable:
    print("EOFTestCase::test_line_continuation_EOF_from_file_bpo2180: skipped")
    raise SystemExit(0)

with os_helper.temp_dir() as temp_dir:
    file_name = script_helper.make_script(temp_dir, "foo", "\\")
    _rc, _out, err = script_helper.assert_python_failure("-X", "utf8", file_name)
    lines = err.decode().splitlines()
    assert lines[-2:] == ["    \\", "SyntaxError: unexpected EOF while parsing"]
    assert lines[-3][-8:] == ", line 1", lines

    file_name = script_helper.make_script(temp_dir, "foo", "\u00e4 = 6\\")
    _rc, _out, err = script_helper.assert_python_failure("-X", "utf8", file_name)
    lines = err.decode().splitlines()
    assert lines[-3:] == ["    \u00e4 = 6\\", "          ^", "SyntaxError: unexpected EOF while parsing"]
    assert lines[-4][-8:] == ", line 1", lines

    file_name = script_helper.make_script(
        temp_dir,
        "foo",
        "# coding:latin1\n\u00e4 = 7\\".encode("latin1"),
    )
    _rc, _out, err = script_helper.assert_python_failure("-X", "utf8", file_name)
    lines = err.decode().splitlines()
    assert lines[-3:] == ["    \u00e4 = 7\\", "          ^", "SyntaxError: unexpected EOF while parsing"]
    assert lines[-4][-8:] == ", line 2", lines

    file_name = script_helper.make_script(temp_dir, "foo", BOM_UTF8 + "\u00e4 = 8\\".encode())
    _rc, _out, err = script_helper.assert_python_failure("-X", "utf8", file_name)
    lines = err.decode().splitlines()
    assert lines[-3:] == ["    \u00e4 = 8\\", "          ^", "SyntaxError: unexpected EOF while parsing"]
    assert lines[-4][-8:] == ", line 1", lines

print("EOFTestCase::test_line_continuation_EOF_from_file_bpo2180: ok")
