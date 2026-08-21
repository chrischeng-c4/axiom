# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "type"
# case = "Formatter__formatException__ei_as__SysExcInfoType_wrong"
# subject = "logging.Formatter.formatException(ei: _SysExcInfoType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.Formatter.formatException(ei: _SysExcInfoType); call it with the wrong type.

typeshed contract: ei is _SysExcInfoType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging import Formatter
obj = object.__new__(Formatter)
try:
    obj.formatException(_W())  # ei: _SysExcInfoType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
