# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "compile_dir_compiles_all_py_files"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: compile_dir returns True and produces one .pyc under __pycache__ for every .py file in the directory"""
import compileall
import os
import tempfile

with tempfile.TemporaryDirectory() as d:
    names = ["alpha.py", "beta.py", "gamma.py"]
    for name in names:
        with open(os.path.join(d, name), "w") as f:
            f.write("x = %d\n" % len(name))
    ok = compileall.compile_dir(d, quiet=2)
    assert ok, "compile_dir returns True"
    cache = os.path.join(d, "__pycache__")
    pycs = [f for f in os.listdir(cache) if f.endswith(".pyc")]
    assert len(pycs) == 3, pycs
print("compile_dir_compiles_all_py_files OK")
