# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "maxlevels_bounds_recursion_depth"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: maxlevels bounds compile_dir recursion: a depth-3 source is left uncompiled at maxlevels=2 but compiled at maxlevels=3"""
import compileall
import importlib.util
import os
import shutil
import tempfile

with tempfile.TemporaryDirectory() as d:
    top = os.path.join(d, "top.py")
    with open(top, "w") as f:
        f.write("x = 1\n")
    path = d
    for i in range(1, 4):
        path = os.path.join(path, "dir_%d" % i)
        os.mkdir(path)
        shutil.copyfile(top, os.path.join(path, "script.py"))
    deep_src = os.path.join(path, "script.py")
    deep_pyc = importlib.util.cache_from_source(deep_src)

    compileall.compile_dir(d, quiet=True, maxlevels=2)
    assert not os.path.isfile(deep_pyc), "depth-3 file untouched at maxlevels=2"

    compileall.compile_dir(d, quiet=True, maxlevels=3)
    assert os.path.isfile(deep_pyc), "depth-3 file compiled at maxlevels=3"
print("maxlevels_bounds_recursion_depth OK")
