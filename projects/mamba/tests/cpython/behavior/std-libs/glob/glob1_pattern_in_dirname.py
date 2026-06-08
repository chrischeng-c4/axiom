# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "behavior"
# case = "glob1_pattern_in_dirname"
# subject = "glob.glob1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""glob.glob1: glob1(dirname, pattern) returns the basenames inside dirname matching pattern: '*.txt' yields only the .txt basenames, '*.rs' only the .rs ones"""
import glob
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    for name in ("alpha.txt", "beta.txt", "gamma.rs", "delta.md"):
        with open(os.path.join(d, name), "w") as fh:
            fh.write("")
    assert sorted(glob.glob1(d, "*.txt")) == ["alpha.txt", "beta.txt"], "glob1 *.txt"
    assert sorted(glob.glob1(d, "*.rs")) == ["gamma.rs"], "glob1 *.rs"

print("glob1_pattern_in_dirname OK")
