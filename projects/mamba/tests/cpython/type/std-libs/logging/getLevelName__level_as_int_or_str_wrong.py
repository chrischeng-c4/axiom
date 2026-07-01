# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "type"
# case = "getLevelName__level_as_int_or_str_wrong"
# subject = "logging.getLevelName(level: int | str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.getLevelName(level: int | str); call it with the wrong type.

typeshed contract: level is int | str across overloads. mamba is force-typed, so a
wrong-typed argument MUST raise TypeError."""

from logging import getLevelName
try:
    getLevelName(3.14)  # level: int | str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
