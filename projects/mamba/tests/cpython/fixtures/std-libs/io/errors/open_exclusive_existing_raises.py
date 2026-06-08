# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "open_exclusive_existing_raises"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: open(path, 'x') on an already-existing file raises FileExistsError"""
import io

import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    path = os.path.join(d, "data.txt")
    with open(path, "w", encoding="utf-8"):
        pass
    raised = False
    try:
        open(path, "x", encoding="utf-8")
    except FileExistsError:
        raised = True
    assert raised, "exclusive-create on existing file must raise FileExistsError"

print("open_exclusive_existing_raises OK")
