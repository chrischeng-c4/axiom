# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "AbstractBasicAuthHandler__init__password_mgr_as_typed_wrong"
# subject = "urllib.request.AbstractBasicAuthHandler.__init__(password_mgr: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.AbstractBasicAuthHandler.__init__(password_mgr: typed); call it with the wrong type.

typeshed contract: password_mgr is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import AbstractBasicAuthHandler
try:
    AbstractBasicAuthHandler(_W())  # password_mgr: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
