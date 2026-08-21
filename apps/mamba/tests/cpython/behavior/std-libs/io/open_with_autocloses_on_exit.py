# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_with_autocloses_on_exit"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: a 'with open(...)' block auto-closes the file on normal exit (.closed becomes True), across buffer sizes"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for bufsize in (0, 100):
        with open(path, "wb", bufsize) as f:
            f.write(b"xxx")
        assert f.closed, f"not closed after with (bufsize={bufsize})"

print("open_with_autocloses_on_exit OK")
