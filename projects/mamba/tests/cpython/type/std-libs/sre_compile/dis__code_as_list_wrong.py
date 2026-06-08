# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sre_compile"
# dimension = "type"
# case = "dis__code_as_list_wrong"
# subject = "sre_compile.dis(code: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed code"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sre_compile.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed code
# mamba-strict-type: TypeError
"""Type wall: sre_compile.dis(code: list); call it with the wrong type.

typeshed contract: code is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from sre_compile import dis
try:
    dis(12345)  # code: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
