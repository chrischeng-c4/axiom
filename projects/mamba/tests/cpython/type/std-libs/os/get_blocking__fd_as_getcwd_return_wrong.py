# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "type"
# case = "get_blocking__fd_as_getcwd_return_wrong"
# subject = "os.get_blocking(fd: int) fed os.getcwd() -> str"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/os.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""#887: stdlib call RETURN types must flow into inference, not just
argument-side walls. `os.getcwd()` is typeshed-contracted `() -> str`; feeding
its result straight into `os.get_blocking(fd: int)` is a wrong-typed call the
① hook can only catch if the `getcwd()` call expression itself infers as
`str` (not `Any`, which the hook always skips). Before #887 a stdlib call's
result was always `Any`, so this specific mismatch was invisible to the wall.

typeshed contract: fd is int, getcwd() -> str. mamba is force-typed, so a
wrong-typed argument MUST raise TypeError (CPython may accept or raise —
mamba's to enforce)."""

from os import get_blocking, getcwd

try:
    get_blocking(getcwd())  # fd: int <- getcwd() -> str, wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
