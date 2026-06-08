# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "surface"
# case = "all_matches_public_names"
# subject = "argparse.__all__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""argparse.__all__: argparse.__all__ lists every public non-module name and includes the core ArgumentParser/Namespace/FileType/ArgumentError/SUPPRESS exports"""
import argparse

# __all__ lists every public, non-module name exported by argparse.
_public = [
    n
    for n, v in vars(argparse).items()
    if not n.startswith("_") and n != "ngettext" and not isinstance(v, type(argparse))
]
assert sorted(_public) == sorted(argparse.__all__), (
    f"__all__ mismatch: extra={sorted(set(_public) - set(argparse.__all__))!r} "
    f"missing={sorted(set(argparse.__all__) - set(_public))!r}"
)
# Core names that must always be exported.
for _name in ("ArgumentParser", "Namespace", "FileType", "ArgumentError", "SUPPRESS"):
    assert _name in argparse.__all__, f"{_name!r} not in __all__"
print("all_matches_public_names OK")
