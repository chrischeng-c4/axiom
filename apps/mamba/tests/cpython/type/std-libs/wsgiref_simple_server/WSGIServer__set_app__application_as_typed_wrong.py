# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "wsgiref_simple_server"
# dimension = "type"
# case = "WSGIServer__set_app__application_as_typed_wrong"
# subject = "wsgiref.simple_server.WSGIServer.set_app(application: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/wsgiref/simple_server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: wsgiref.simple_server.WSGIServer.set_app(application: typed); call it with the wrong type.

typeshed contract: application is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from wsgiref.simple_server import WSGIServer
obj = object.__new__(WSGIServer)
try:
    obj.set_app(_W())  # application: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
