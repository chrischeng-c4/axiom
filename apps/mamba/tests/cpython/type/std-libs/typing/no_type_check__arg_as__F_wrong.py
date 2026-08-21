# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "type"
# case = "no_type_check__arg_as__F_wrong"
# subject = "typing.no_type_check(arg: _F)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed arg"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/typing.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed arg
# mamba-strict-type: TypeError
"""Type wall: typing.no_type_check(arg: _F); call it with the wrong type.

typeshed contract: arg is _F. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from typing import no_type_check
try:
    no_type_check(_W())  # arg: _F <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
