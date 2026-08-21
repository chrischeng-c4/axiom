# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "logging_handlers"
# dimension = "type"
# case = "SysLogHandler__encodePriority__facility_as_typed_wrong"
# subject = "logging.handlers.SysLogHandler.encodePriority(facility: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/logging/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: logging.handlers.SysLogHandler.encodePriority(facility: typed); call it with the wrong type.

typeshed contract: facility is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from logging.handlers import SysLogHandler
obj = object.__new__(SysLogHandler)
try:
    obj.encodePriority(_W(), None)  # facility: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
