# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "type"
# case = "timerfd_settime_ns__fd_as_FileDescriptor_wrong"
# subject = "os.timerfd_settime_ns(fd: FileDescriptor)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/os.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: os.timerfd_settime_ns(fd: FileDescriptor); call it with the wrong type.

typeshed contract: fd is FileDescriptor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from os import timerfd_settime_ns
try:
    timerfd_settime_ns(_W())  # fd: FileDescriptor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
