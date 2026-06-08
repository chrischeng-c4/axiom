# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "open_closefd_true_for_filename"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: a filename open has buffer.raw.closefd True; a borrowed-fd open (closefd=False) has it False and reads the same content"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w", encoding="utf-8") as f:
        f.write("egg\n")
    with open(path, "r", encoding="utf-8") as f:
        assert f.buffer.raw.closefd is True, "filename open closefd"
        borrowed = open(f.fileno(), "r", encoding="utf-8", closefd=False)
        assert borrowed.buffer.raw.closefd is False, "borrowed fd closefd"
        assert borrowed.read() == "egg\n", "read via borrowed fd"
        borrowed.close()
        read_after_close = False
        try:
            borrowed.read()
        except ValueError:
            read_after_close = True
        assert read_after_close, "read after close did not raise ValueError"

print("open_closefd_true_for_filename OK")
