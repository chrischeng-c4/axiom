# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "compile_file_legacy_writes_pyc_next_to_source"
# subject = "compileall.compile_file"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""compileall.compile_file: compile_file with legacy=True writes the .pyc next to the .py source (legacy layout) instead of under __pycache__"""
import compileall
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "legacy_test.py")
    with open(src, "w") as f:
        f.write("z = 99\n")
    ok = compileall.compile_file(src, quiet=2, legacy=True)
    assert ok, "legacy compile succeeds"
    # Legacy layout places the cache file beside the source, not in __pycache__.
    pyc_next_to = os.path.join(d, "legacy_test.pyc")
    assert os.path.exists(pyc_next_to), os.listdir(d)
    assert not os.path.exists(os.path.join(d, "__pycache__")), os.listdir(d)
print("compile_file_legacy_writes_pyc_next_to_source OK")
