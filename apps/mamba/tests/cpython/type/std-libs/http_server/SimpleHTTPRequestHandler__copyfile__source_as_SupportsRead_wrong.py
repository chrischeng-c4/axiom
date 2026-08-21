# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "http_server"
# dimension = "type"
# case = "SimpleHTTPRequestHandler__copyfile__source_as_SupportsRead_wrong"
# subject = "http.server.SimpleHTTPRequestHandler.copyfile(source: SupportsRead)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/http/server.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: http.server.SimpleHTTPRequestHandler.copyfile(source: SupportsRead); call it with the wrong type.

typeshed contract: source is SupportsRead. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from http.server import SimpleHTTPRequestHandler
obj = object.__new__(SimpleHTTPRequestHandler)
try:
    obj.copyfile(_W(), None)  # source: SupportsRead <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
