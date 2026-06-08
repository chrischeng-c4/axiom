# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "real_world"
# case = "symlink_limit_filters_by_dest_tree"
# subject = "compileall.compile_dir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compileall.py"
# status = "filled"
# ///
"""compileall.compile_dir: a build that compiles a directory of symlinks with limit_sl_dest=<allowed tree>: symlinks resolving inside the allowed tree are compiled, symlinks resolving elsewhere are skipped"""
import compileall
import importlib.util
import os
import tempfile

# Realistic packaging scenario: a staging dir of symlinks is compiled, but a
# build wants to compile only links pointing into one allowed source tree.
# Everything lives under one TemporaryDirectory so the symlinks are local.
with tempfile.TemporaryDirectory() as d:
    allowed = os.path.join(d, "allowed")
    prohibited = os.path.join(d, "prohibited")
    symlinks = os.path.join(d, "symlinks")
    os.makedirs(allowed)
    os.makedirs(prohibited)
    os.makedirs(symlinks)

    allowed_src = os.path.join(allowed, "a.py")
    prohibited_src = os.path.join(prohibited, "p.py")
    with open(allowed_src, "w") as f:
        f.write("a = 0\n")
    with open(prohibited_src, "w") as f:
        f.write("p = 0\n")

    allowed_link = os.path.join(symlinks, "a.py")
    prohibited_link = os.path.join(symlinks, "p.py")
    os.symlink(allowed_src, allowed_link)
    os.symlink(prohibited_src, prohibited_link)

    allowed_pyc = importlib.util.cache_from_source(allowed_link)
    prohibited_pyc = importlib.util.cache_from_source(prohibited_link)

    compileall.compile_dir(symlinks, quiet=True, limit_sl_dest=allowed)

    assert os.path.isfile(allowed_pyc), "symlink into allowed tree compiled"
    assert not os.path.isfile(prohibited_pyc), "symlink outside allowed tree skipped"
print("symlink_limit_filters_by_dest_tree OK")
