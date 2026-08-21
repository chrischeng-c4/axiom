# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "shutil"
# dimension = "type"
# case = "chown__path_as_FileDescriptorOrPath_wrong"
# subject = "shutil.chown(path: FileDescriptorOrPath)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed path"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/shutil.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed path
# mamba-strict-type: TypeError
"""Type wall: shutil.chown(path: FileDescriptorOrPath); call it with the wrong type.

typeshed contract: path is FileDescriptorOrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from shutil import chown
try:
    chown(_W(), None)  # path: FileDescriptorOrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
