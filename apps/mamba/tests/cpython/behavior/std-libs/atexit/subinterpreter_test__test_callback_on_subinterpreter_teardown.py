# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "subinterpreter_test__test_callback_on_subinterpreter_teardown"
# subject = "cpython.test_atexit.SubinterpreterTest.test_callback_on_subinterpreter_teardown"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: SubinterpreterTest::test_callback_on_subinterpreter_teardown (CPython 3.12 oracle)."""

import os
import textwrap
from test import support


expected = b"The test has passed!"
read_fd, write_fd = os.pipe()
try:
    code = textwrap.dedent(
        r"""
        import os
        import atexit
        def callback():
            os.write({:d}, b"The test has passed!")
        atexit.register(callback)
        """.format(write_fd)
    )
    ret = support.run_in_subinterp(code)
    os.close(write_fd)
    write_fd = None
    assert ret == 0, ret
    assert os.read(read_fd, len(expected)) == expected
finally:
    if write_fd is not None:
        os.close(write_fd)
    os.close(read_fd)

print("SubinterpreterTest::test_callback_on_subinterpreter_teardown: ok")
