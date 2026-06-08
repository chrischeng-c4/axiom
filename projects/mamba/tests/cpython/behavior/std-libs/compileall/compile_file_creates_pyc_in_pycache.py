# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "compile_file_creates_pyc_in_pycache"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_file: compile_file on a valid .py returns True and creates a matching .pyc under __pycache__ whose name starts with the module name"""
import compileall
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "hello.py")
    with open(src, "w") as f:
        f.write("print('hello')\n")
    ok = compileall.compile_file(src, quiet=2)
    assert ok, "compile_file returns True"
    cache = os.path.join(d, "__pycache__")
    assert os.path.isdir(cache), "__pycache__ created"
    pycs = [f for f in os.listdir(cache) if f.endswith(".pyc")]
    assert len(pycs) >= 1, pycs
    # The cache file name is derived from the module name.
    assert any(f.startswith("hello.") for f in pycs), pycs
print("compile_file_creates_pyc_in_pycache OK")
