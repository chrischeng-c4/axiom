# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_filelist"
# dimension = "type"
# case = "FileList__exclude_pattern__pattern_as_str_wrong"
# subject = "distutils.filelist.FileList.exclude_pattern(pattern: str)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed pattern"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/filelist.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed pattern
# mamba-strict-type: TypeError
"""Type wall: distutils.filelist.FileList.exclude_pattern(pattern: str); call it with the wrong type.

typeshed contract: pattern is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.filelist import FileList
obj = object.__new__(FileList)
try:
    obj.exclude_pattern(12345)  # pattern: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
