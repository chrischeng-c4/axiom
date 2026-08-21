# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "behavior"
# case = "pathlib_path_inputs_accepted"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: pathlib.Path arguments are accepted everywhere a str path is: compile_dir(Path(d), prependdir=Path(...)) and compile_file(Path(src), stripdir=Path(...)) both succeed and create the cache file"""
import compileall
import importlib.util
import os
import pathlib
import tempfile

# Path inputs to compile_dir + prependdir: still creates the normal cache file.
with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "p.py")
    with open(src, "w") as f:
        f.write("x = 123\n")
    cache = importlib.util.cache_from_source(src)
    assert not os.path.isfile(cache), "cache absent before compile"
    ok = compileall.compile_dir(pathlib.Path(d),
                                prependdir=pathlib.Path("prepend_root"),
                                quiet=2)
    assert ok, "compile_dir with Path + prependdir succeeds"
    assert os.path.isfile(cache), "cache created"

# Path inputs to compile_file + stripdir on a single file.
with tempfile.TemporaryDirectory() as d:
    src = os.path.join(d, "s.py")
    with open(src, "w") as f:
        f.write("y = 1\n")
    cache = importlib.util.cache_from_source(src)
    ok = compileall.compile_file(pathlib.Path(src),
                                 stripdir=pathlib.Path("strip_root"),
                                 quiet=2)
    assert ok, "compile_file with Path + stripdir succeeds"
    assert os.path.isfile(cache), "cache created"
print("pathlib_path_inputs_accepted OK")
