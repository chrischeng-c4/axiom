# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "genericpath"
# dimension = "type"
# case = "samefile__f1_as_FileDescriptorOrPath_wrong"
# subject = "genericpath.samefile(f1: FileDescriptorOrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/genericpath.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: genericpath.samefile(f1: FileDescriptorOrPath); call it with the wrong type.

typeshed contract: f1 is FileDescriptorOrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from genericpath import samefile
try:
    samefile(_W(), None)  # f1: FileDescriptorOrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
