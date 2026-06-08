# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "type"
# case = "getmembers__predicate_as__GetMembersPredicateTypeGuard_wrong"
# subject = "inspect.getmembers(predicate: _GetMembersPredicateTypeGuard)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed predicate"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/inspect.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed predicate
# mamba-strict-type: TypeError
"""Type wall: inspect.getmembers(predicate: _GetMembersPredicateTypeGuard); call it with the wrong type.

typeshed contract: predicate is _GetMembersPredicateTypeGuard. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from inspect import getmembers
try:
    getmembers(None, _W())  # predicate: _GetMembersPredicateTypeGuard <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
