# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "non_py_file_skipped_no_pycache"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_file: compile_file on a non-.py file is a no-op: it never creates a __pycache__ directory"""
import compileall
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    data = os.path.join(d, "file")
    with open(data, "wb"):
        pass
    compileall.compile_file(data, quiet=2)
    assert not os.path.exists(os.path.join(d, "__pycache__")), os.listdir(d)
print("non_py_file_skipped_no_pycache OK")
