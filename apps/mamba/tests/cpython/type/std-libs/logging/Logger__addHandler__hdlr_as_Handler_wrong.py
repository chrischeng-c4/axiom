# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "type"
# case = "Logger__addHandler__hdlr_as_Handler_wrong"
# subject = "logging.Logger.addHandler(hdlr: Handler)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.Logger.addHandler(hdlr: Handler); call it with the wrong type.

typeshed contract: hdlr is Handler. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging import Logger
obj = object.__new__(Logger)
try:
    obj.addHandler(_W())  # hdlr: Handler <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
