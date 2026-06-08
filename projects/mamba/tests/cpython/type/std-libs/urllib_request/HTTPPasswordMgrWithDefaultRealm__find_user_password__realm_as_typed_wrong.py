# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "type"
# case = "HTTPPasswordMgrWithDefaultRealm__find_user_password__realm_as_typed_wrong"
# subject = "urllib.request.HTTPPasswordMgrWithDefaultRealm.find_user_password(realm: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/request.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.request.HTTPPasswordMgrWithDefaultRealm.find_user_password(realm: typed); call it with the wrong type.

typeshed contract: realm is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.request import HTTPPasswordMgrWithDefaultRealm
obj = object.__new__(HTTPPasswordMgrWithDefaultRealm)
try:
    obj.find_user_password(_W(), "")  # realm: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
