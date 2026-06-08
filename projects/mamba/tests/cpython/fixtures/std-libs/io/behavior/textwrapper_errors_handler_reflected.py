# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "behavior"
# case = "textwrapper_errors_handler_reflected"
# subject = "io.TextIOWrapper"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.TextIOWrapper: TextIOWrapper.errors reflects the error handler chosen at open time (default 'strict', or 'replace')"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w", encoding="utf-8") as f:
        assert f.errors == "strict", f"default errors = {f.errors!r}"
    with open(path, "w", encoding="utf-8", errors="replace") as f:
        assert f.errors == "replace", f"errors = {f.errors!r}"

print("textwrapper_errors_handler_reflected OK")
