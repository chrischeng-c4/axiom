# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "compile_dir_rx_skips_matching_files"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: compile_dir with rx=<compiled regex> skips files whose path matches the pattern and compiles only the rest"""
import compileall
import os
import re
import tempfile

with tempfile.TemporaryDirectory() as d:
    for fn in ["include.py", "skip_me.py", "skip_too.py"]:
        with open(os.path.join(d, fn), "w") as f:
            f.write("pass\n")
    compileall.compile_dir(d, quiet=2, rx=re.compile(r"skip.*\.py"))
    cache = os.path.join(d, "__pycache__")
    pycs = [f for f in os.listdir(cache) if f.endswith(".pyc")]
    assert all("skip" not in f for f in pycs), pycs
    assert any("include" in f for f in pycs), pycs
print("compile_dir_rx_skips_matching_files OK")
