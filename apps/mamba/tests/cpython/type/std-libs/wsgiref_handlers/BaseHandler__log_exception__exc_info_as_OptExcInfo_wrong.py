# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_handlers"
# dimension = "type"
# case = "BaseHandler__log_exception__exc_info_as_OptExcInfo_wrong"
# subject = "wsgiref.handlers.BaseHandler.log_exception(exc_info: OptExcInfo)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.handlers.BaseHandler.log_exception(exc_info: OptExcInfo); call it with the wrong type.

typeshed contract: exc_info is OptExcInfo. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.handlers import BaseHandler
obj = object.__new__(BaseHandler)
try:
    obj.log_exception(_W())  # exc_info: OptExcInfo <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
