# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "termios"
# dimension = "type"
# case = "tcsetattr__fd_as_FileDescriptorLike_wrong"
# subject = "termios.tcsetattr(fd: FileDescriptorLike)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/termios.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: termios.tcsetattr(fd: FileDescriptorLike); call it with the wrong type.

typeshed contract: fd is FileDescriptorLike. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from termios import tcsetattr
try:
    tcsetattr(_W(), 0, None)  # fd: FileDescriptorLike <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
