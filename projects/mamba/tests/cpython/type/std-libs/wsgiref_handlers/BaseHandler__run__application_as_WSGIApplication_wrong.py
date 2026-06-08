# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_handlers"
# dimension = "type"
# case = "BaseHandler__run__application_as_WSGIApplication_wrong"
# subject = "wsgiref.handlers.BaseHandler.run(application: WSGIApplication)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/handlers.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.handlers.BaseHandler.run(application: WSGIApplication); call it with the wrong type.

typeshed contract: application is WSGIApplication. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.handlers import BaseHandler
obj = object.__new__(BaseHandler)
try:
    obj.run(_W())  # application: WSGIApplication <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
