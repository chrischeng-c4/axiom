# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SMTPHandler__getSubject__record_as_LogRecord_wrong"
# subject = "logging.handlers.SMTPHandler.getSubject(record: LogRecord)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SMTPHandler.getSubject(record: LogRecord); call it with the wrong type.

typeshed contract: record is LogRecord. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import SMTPHandler
obj = object.__new__(SMTPHandler)
try:
    obj.getSubject(_W())  # record: LogRecord <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
