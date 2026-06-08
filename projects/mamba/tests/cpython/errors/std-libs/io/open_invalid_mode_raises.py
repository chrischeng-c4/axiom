# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "open_invalid_mode_raises"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: open() with a nonsensical mode string ('rwax+') raises ValueError"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    raised = False
    try:
        open(path, "rwax+", encoding="utf-8")
    except ValueError:
        raised = True
    assert raised, "nonsensical mode string must raise ValueError"

print("open_invalid_mode_raises OK")
