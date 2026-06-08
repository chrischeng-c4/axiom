# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "temporary_directory_dir_nests_child"
# subject = "tempfile.TemporaryDirectory"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.TemporaryDirectory: TemporaryDirectory(dir=parent) creates the child directory under the given parent path"""
import os
import tempfile

with tempfile.TemporaryDirectory() as _parent:
    with tempfile.TemporaryDirectory(dir=_parent) as _child:
        assert _child.startswith(_parent), f"child under parent: {_child!r}"
        assert os.path.isdir(_child), "child is dir"
print("temporary_directory_dir_nests_child OK")
