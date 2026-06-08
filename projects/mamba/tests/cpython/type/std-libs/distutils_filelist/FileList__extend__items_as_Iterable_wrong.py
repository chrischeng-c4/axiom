# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_filelist"
# dimension = "type"
# case = "FileList__extend__items_as_Iterable_wrong"
# subject = "distutils.filelist.FileList.extend(items: Iterable)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/filelist.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.filelist.FileList.extend(items: Iterable); call it with the wrong type.

typeshed contract: items is Iterable. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.filelist import FileList
obj = object.__new__(FileList)
try:
    obj.extend(_W())  # items: Iterable <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
