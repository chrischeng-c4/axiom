# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "path_splitext_rule"
# subject = "os.path.splitext"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.path.splitext: os.path.splitext splits the final extension only: file.txt, file.tar.gz, noext, .hidden cases"""
import os.path

cases = [
    ("file.txt", ("file", ".txt")),
    ("file.tar.gz", ("file.tar", ".gz")),
    ("noext", ("noext", "")),
    (".hidden", (".hidden", "")),
]
for inp, expected in cases:
    got = os.path.splitext(inp)
    assert got == expected, f"splitext({inp!r}) = {got!r}, expected {expected!r}"
print("path_splitext_rule OK")
