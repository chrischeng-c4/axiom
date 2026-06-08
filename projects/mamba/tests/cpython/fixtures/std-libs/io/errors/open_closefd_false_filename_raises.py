# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "open_closefd_false_filename_raises"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: open(path, closefd=False) raises ValueError because closefd=False is only valid for an existing fd, not a filename"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    for mode in ("w", "r"):
        if mode == "r":
            with open(path, "w", encoding="utf-8"):
                pass
        raised = False
        try:
            open(path, mode, encoding="utf-8", closefd=False)
        except ValueError:
            raised = True
        assert raised, f"closefd=False on filename ({mode}) must raise ValueError"

print("open_closefd_false_filename_raises OK")
