# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "type"
# case = "dump_c_stack__file_as_FileDescriptorLike_wrong"
# subject = "faulthandler.dump_c_stack(file: FileDescriptorLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/faulthandler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: faulthandler.dump_c_stack(file: FileDescriptorLike); call it with the wrong type.

typeshed contract: file is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from faulthandler import dump_c_stack
try:
    dump_c_stack(_W())  # file: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
