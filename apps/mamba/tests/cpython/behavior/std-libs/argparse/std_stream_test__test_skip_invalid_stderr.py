# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "behavior"
# case = "std_stream_test__test_skip_invalid_stderr"
# subject = "cpython.test_argparse.StdStreamTest.test_skip_invalid_stderr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_argparse.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_argparse.py::StdStreamTest::test_skip_invalid_stderr
"""Auto-ported test: StdStreamTest::test_skip_invalid_stderr."""


import argparse
import contextlib
from unittest import mock


parser = argparse.ArgumentParser()
with contextlib.redirect_stderr(None), mock.patch("argparse._sys.exit") as exit_mock:
    parser.exit(status=0, message="foo")

exit_mock.assert_called_once_with(0)

print("StdStreamTest::test_skip_invalid_stderr: ok")
