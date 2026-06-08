# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "with_suffix_name_stem"
# subject = "pathlib.Path"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.Path: with_suffix/.with_name/.with_stem return new paths: file.txt.with_suffix('.csv')==file.csv, /tmp/old.txt.with_name('new.csv')==/tmp/new.csv, /a/b.tar.gz.with_stem('d')==/a/d.gz"""
import pathlib

Path = pathlib.Path

_new_ext = Path("file.txt").with_suffix(".csv")
assert _new_ext == Path("file.csv"), f"with_suffix = {_new_ext!r}"
_new_name = Path("/tmp/old.txt").with_name("new.csv")
assert _new_name == Path("/tmp/new.csv"), f"with_name = {_new_name!r}"
_new_stem = Path("/a/b.tar.gz").with_stem("d")
assert _new_stem == Path("/a/d.gz"), f"with_stem = {_new_stem!r}"
print("with_suffix_name_stem OK")
