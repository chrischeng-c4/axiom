# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "errors"
# case = "utime_short_ns_raises_typeerror"
# subject = "os.utime"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_os.py"
# status = "filled"
# ///
"""os.utime: os.utime(path, ns=(5,)) rejects a wrong-length ns tuple with TypeError, against a real temp file"""
import os
import tempfile

with tempfile.TemporaryDirectory() as td:
    fpath = os.path.join(td, "f")
    with open(fpath, "w", encoding="utf-8") as f:
        f.write("")
    raised = False
    try:
        os.utime(fpath, ns=(5,))
    except TypeError:
        raised = True
    assert raised, "utime with a wrong-length ns tuple should raise TypeError"
print("utime_short_ns_raises_typeerror OK")
