# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "contextlib"
# dimension = "type"
# case = "chdir__init__path_as__T_fd_or_any_path_wrong"
# subject = "contextlib.chdir.__init__(path: _T_fd_or_any_path)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/contextlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: contextlib.chdir.__init__(path: _T_fd_or_any_path); call it with the wrong type.

typeshed contract: path is _T_fd_or_any_path. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from contextlib import chdir
try:
    chdir(_W())  # path: _T_fd_or_any_path <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
