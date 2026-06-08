# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "errors"
# case = "open_nul_in_path_raises"
# subject = "io.open"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_io.py"
# status = "filled"
# ///
"""io.open: open() of a path containing an embedded NUL byte raises ValueError"""
import io

raised = False
try:
    open("foo\x00bar", "w", encoding="utf-8")
except ValueError:
    raised = True
assert raised, "embedded NUL byte in path must raise ValueError"

print("open_nul_in_path_raises OK")
