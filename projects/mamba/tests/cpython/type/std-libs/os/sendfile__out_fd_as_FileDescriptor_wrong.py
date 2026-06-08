# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "type"
# case = "sendfile__out_fd_as_FileDescriptor_wrong"
# subject = "os.sendfile(out_fd: FileDescriptor)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed out_fd"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/os.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed out_fd
# mamba-strict-type: TypeError
"""Type wall: os.sendfile(out_fd: FileDescriptor); call it with the wrong type.

typeshed contract: out_fd is FileDescriptor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from os import sendfile
try:
    sendfile(_W(), None, None, 0)  # out_fd: FileDescriptor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
