# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_config"
# dimension = "type"
# case = "DictConfigurator__common_logger_config__logger_as_Logger_wrong"
# subject = "logging.config.DictConfigurator.common_logger_config(logger: Logger)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/config.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.config.DictConfigurator.common_logger_config(logger: Logger); call it with the wrong type.

typeshed contract: logger is Logger. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.config import DictConfigurator
obj = object.__new__(DictConfigurator)
try:
    obj.common_logger_config(_W(), None)  # logger: Logger <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
