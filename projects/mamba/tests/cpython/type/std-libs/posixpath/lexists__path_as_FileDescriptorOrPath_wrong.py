# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "type"
# case = "lexists__path_as_FileDescriptorOrPath_wrong"
# subject = "posixpath.lexists(path: FileDescriptorOrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/posixpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: posixpath.lexists(path: FileDescriptorOrPath); call it with the wrong type.

typeshed contract: path is FileDescriptorOrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from posixpath import lexists
try:
    lexists(_W())  # path: FileDescriptorOrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
