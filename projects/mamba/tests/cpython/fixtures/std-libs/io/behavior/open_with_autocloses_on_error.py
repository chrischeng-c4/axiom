# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_with_autocloses_on_error"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: a 'with open(...)' block auto-closes the file even when the body raises, across buffer sizes"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for bufsize in (0, 100):
        raised = False
        try:
            with open(path, "wb", bufsize) as f:
                raise ZeroDivisionError
        except ZeroDivisionError:
            raised = True
        assert raised, "exception swallowed"
        assert f.closed, f"not closed after error (bufsize={bufsize})"

print("open_with_autocloses_on_error OK")
