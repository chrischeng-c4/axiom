# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_windows_utils"
# dimension = "type"
# case = "Popen____new____args_as__CMD_wrong"
# subject = "asyncio.windows_utils.Popen.__new__(args: _CMD)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/windows_utils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.windows_utils.Popen.__new__(args: _CMD); call it with the wrong type.

typeshed contract: args is _CMD. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.windows_utils import Popen
obj = object.__new__(Popen)
try:
    obj.__new__(_W())  # args: _CMD <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
