# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging"
# dimension = "type"
# case = "Formatter__formatTime__record_as_LogRecord_wrong"
# subject = "logging.Formatter.formatTime(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.Formatter.formatTime(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging import Formatter
obj = object.__new__(Formatter)
try:
    obj.formatTime(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
