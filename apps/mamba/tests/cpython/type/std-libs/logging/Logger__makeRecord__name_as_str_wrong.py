# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "type"
# case = "Logger__makeRecord__name_as_str_wrong"
# subject = "logging.Logger.makeRecord(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.Logger.makeRecord(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging import Logger
obj = object.__new__(Logger)
try:
    obj.makeRecord(12345, 0, "", 0, None, None, None)  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
