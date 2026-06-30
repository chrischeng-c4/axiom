# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ast"
# dimension = "type"
# case = "Delete__init__targets_as_list_wrong"
# subject = "ast.Delete.__init__(targets: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ast.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: ast.Delete.__init__(targets: list); call it with the wrong type.

typeshed contract: targets is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ast import Delete
try:
    Delete(12345)  # targets: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
