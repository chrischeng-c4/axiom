# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "type"
# case = "enable__file_as_FileDescriptorLike_wrong"
# subject = "faulthandler.enable(file: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/faulthandler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: faulthandler.enable(file: FileDescriptorLike); call it with the wrong type.

typeshed contract: file is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from faulthandler import enable
try:
    enable(_W())  # file: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
