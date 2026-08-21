# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "argparse"
# dimension = "type"
# case = "FileType____call____string_as_str_wrong"
# subject = "argparse.FileType.__call__(string: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/argparse.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: argparse.FileType.__call__(string: str); call it with the wrong type.

typeshed contract: string is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from argparse import FileType
obj = object.__new__(FileType)
try:
    obj.__call__(12345)  # string: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
