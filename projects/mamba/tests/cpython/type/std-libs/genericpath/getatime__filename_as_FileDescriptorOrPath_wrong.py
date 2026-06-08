# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericpath"
# dimension = "type"
# case = "getatime__filename_as_FileDescriptorOrPath_wrong"
# subject = "genericpath.getatime(filename: FileDescriptorOrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/genericpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: genericpath.getatime(filename: FileDescriptorOrPath); call it with the wrong type.

typeshed contract: filename is FileDescriptorOrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from genericpath import getatime
try:
    getatime(_W())  # filename: FileDescriptorOrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
