# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "FileHandler__open_local_file__req_as_Request_wrong"
# subject = "urllib.request.FileHandler.open_local_file(req: Request)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.FileHandler.open_local_file(req: Request); call it with the wrong type.

typeshed contract: req is Request. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import FileHandler
obj = object.__new__(FileHandler)
try:
    obj.open_local_file(_W())  # req: Request <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
