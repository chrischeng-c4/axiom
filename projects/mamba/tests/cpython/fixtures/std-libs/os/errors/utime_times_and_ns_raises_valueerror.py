# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "utime_times_and_ns_raises_valueerror"
# subject = "os.utime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.utime: os.utime(path, (5, 5), ns=(5, 5)) rejects supplying both a times tuple and ns= at once with ValueError, against a real temp file"""
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    fpath = os.path.join(td, "f")
    with open(fpath, "w", encoding="utf-8") as f:
        f.write("")
    raised = False
    try:
        os.utime(fpath, (5, 5), ns=(5, 5))
    except ValueError:
        raised = True
    assert raised, "utime with both times and ns= should raise ValueError"
print("utime_times_and_ns_raises_valueerror OK")
