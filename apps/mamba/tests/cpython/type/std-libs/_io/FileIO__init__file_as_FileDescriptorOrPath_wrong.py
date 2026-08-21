# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_io"
# dimension = "type"
# case = "FileIO__init__file_as_FileDescriptorOrPath_wrong"
# subject = "_io.FileIO.__init__(file: FileDescriptorOrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_io.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _io.FileIO.__init__(file: FileDescriptorOrPath); call it with the wrong type.

typeshed contract: file is FileDescriptorOrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _io import FileIO
try:
    FileIO(_W())  # file: FileDescriptorOrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
