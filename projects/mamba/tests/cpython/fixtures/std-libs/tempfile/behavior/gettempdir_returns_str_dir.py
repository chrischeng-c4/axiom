# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "gettempdir_returns_str_dir"
# subject = "tempfile.gettempdir"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.gettempdir: gettempdir() returns a non-empty str that names an existing directory"""
import os
import tempfile

_tmpdir = tempfile.gettempdir()
assert isinstance(_tmpdir, str), f"gettempdir type = {type(_tmpdir)!r}"
assert len(_tmpdir) > 0, "gettempdir non-empty"
assert os.path.isdir(_tmpdir), f"tmpdir is dir: {_tmpdir!r}"
print("gettempdir_returns_str_dir OK")
