# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_filelist"
# dimension = "type"
# case = "FileList__findall__dir_as_str_wrong"
# subject = "distutils.filelist.FileList.findall(dir: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/filelist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.filelist.FileList.findall(dir: str); call it with the wrong type.

typeshed contract: dir is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from distutils.filelist import FileList
obj = object.__new__(FileList)
try:
    obj.findall(12345)  # dir: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
