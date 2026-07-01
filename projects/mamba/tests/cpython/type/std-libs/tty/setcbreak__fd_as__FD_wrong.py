# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tty"
# dimension = "type"
# case = "setcbreak__fd_as__FD_wrong"
# subject = "tty.setcbreak(fd: _FD)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/tty.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: tty.setcbreak(fd: _FD); call it with the wrong type.

typeshed contract: fd is _FD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from tty import setcbreak
try:
    setcbreak(_W())  # fd: _FD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
