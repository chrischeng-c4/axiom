# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ipaddress"
# dimension = "type"
# case = "get_mixed_type_key__obj_as__A_wrong"
# subject = "ipaddress.get_mixed_type_key(obj: _A)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed obj"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ipaddress.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed obj
# mamba-strict-type: TypeError
"""Type wall: ipaddress.get_mixed_type_key(obj: _A); call it with the wrong type.

typeshed contract: obj is _A. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from ipaddress import get_mixed_type_key
try:
    get_mixed_type_key(_W())  # obj: _A <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
