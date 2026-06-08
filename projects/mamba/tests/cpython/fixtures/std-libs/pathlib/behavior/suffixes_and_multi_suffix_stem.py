# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "suffixes_and_multi_suffix_stem"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: Path('file.tar.gz').suffixes == ['.tar', '.gz'] and its stem keeps all but the last suffix ('file.tar')"""
import pathlib

Path = pathlib.Path

_multi = Path("file.tar.gz")
assert _multi.suffixes == [".tar", ".gz"], f"suffixes = {_multi.suffixes!r}"
assert _multi.stem == "file.tar", f"multi stem = {_multi.stem!r}"
print("suffixes_and_multi_suffix_stem OK")
