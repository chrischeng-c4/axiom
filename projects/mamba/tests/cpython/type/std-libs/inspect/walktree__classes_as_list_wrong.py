# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "type"
# case = "walktree__classes_as_list_wrong"
# subject = "inspect.walktree(classes: list)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/inspect.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: inspect.walktree(classes: list); call it with the wrong type.

typeshed contract: classes is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from inspect import walktree
try:
    walktree(12345, None, None)  # classes: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
