# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "results_include_directories"
# subject = "glob.glob"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob: glob('*') includes both files and subdirectories: a temp dir with a file 'file.txt' and a subdir 'subdir' yields both basenames"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    with open(os.path.join(d, "file.txt"), "w") as fh:
        fh.write("")
    os.mkdir(os.path.join(d, "subdir"))
    results = sorted(glob.glob(os.path.join(d, "*")))
    bases = [os.path.basename(p) for p in results]
    assert "file.txt" in bases, f"file in results = {bases!r}"
    assert "subdir" in bases, f"dir in results = {bases!r}"

print("results_include_directories OK")
