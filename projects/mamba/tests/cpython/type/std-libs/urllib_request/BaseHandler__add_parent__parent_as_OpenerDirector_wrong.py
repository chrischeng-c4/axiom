# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "BaseHandler__add_parent__parent_as_OpenerDirector_wrong"
# subject = "urllib.request.BaseHandler.add_parent(parent: OpenerDirector)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.BaseHandler.add_parent(parent: OpenerDirector); call it with the wrong type.

typeshed contract: parent is OpenerDirector. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import BaseHandler
obj = object.__new__(BaseHandler)
try:
    obj.add_parent(_W())  # parent: OpenerDirector <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
