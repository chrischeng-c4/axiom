# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "type"
# case = "tcsendbreak__fd_as_FileDescriptorLike_wrong"
# subject = "termios.tcsendbreak(fd: FileDescriptorLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/termios.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: termios.tcsendbreak(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from termios import tcsendbreak
try:
    tcsendbreak(_W(), 0)  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
