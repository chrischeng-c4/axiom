# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "NTEventLogHandler__init__appname_as_str_wrong"
# subject = "logging.handlers.NTEventLogHandler.__init__(appname: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.NTEventLogHandler.__init__(appname: str); call it with the wrong type.

typeshed contract: appname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import NTEventLogHandler
try:
    NTEventLogHandler(12345)  # appname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
