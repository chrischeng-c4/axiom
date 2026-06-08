# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "BaseRotatingHandler__rotation_filename__default_name_as_str_wrong"
# subject = "logging.handlers.BaseRotatingHandler.rotation_filename(default_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.BaseRotatingHandler.rotation_filename(default_name: str); call it with the wrong type.

typeshed contract: default_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from logging.handlers import BaseRotatingHandler
obj = object.__new__(BaseRotatingHandler)
try:
    obj.rotation_filename(12345)  # default_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
