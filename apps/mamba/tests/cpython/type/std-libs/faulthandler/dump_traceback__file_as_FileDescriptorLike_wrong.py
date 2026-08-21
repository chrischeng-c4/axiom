# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "type"
# case = "dump_traceback__file_as_FileDescriptorLike_wrong"
# subject = "faulthandler.dump_traceback(file: FileDescriptorLike)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/faulthandler.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: faulthandler.dump_traceback(file: FileDescriptorLike); call it with the wrong type.

typeshed contract: file is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from faulthandler import dump_traceback
try:
    dump_traceback(_W())  # file: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
