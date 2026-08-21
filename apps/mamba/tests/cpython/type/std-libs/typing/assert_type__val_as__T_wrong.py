# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "type"
# case = "assert_type__val_as__T_wrong"
# subject = "typing.assert_type(val: _T)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed val"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed val
# mamba-strict-type: TypeError
"""Type wall: typing.assert_type(val: _T); call it with the wrong type.

typeshed contract: val is _T. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing import assert_type
try:
    assert_type(_W(), None)  # val: _T <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
