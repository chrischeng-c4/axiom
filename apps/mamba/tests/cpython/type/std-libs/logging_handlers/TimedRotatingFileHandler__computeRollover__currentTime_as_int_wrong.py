# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "TimedRotatingFileHandler__computeRollover__currentTime_as_int_wrong"
# subject = "logging.handlers.TimedRotatingFileHandler.computeRollover(currentTime: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.TimedRotatingFileHandler.computeRollover(currentTime: int); call it with the wrong type.

typeshed contract: currentTime is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import TimedRotatingFileHandler
obj = object.__new__(TimedRotatingFileHandler)
try:
    obj.computeRollover("not_an_int")  # currentTime: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
