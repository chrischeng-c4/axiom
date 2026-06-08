# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "resolve_bases_mro_entries"
# subject = "types.resolve_bases"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.resolve_bases: resolve_bases expands __mro_entries__ but leaves plain bases untouched, returning the same tuple object when nothing needs resolving"""
import types


class P:
    pass


class Q:
    def __mro_entries__(self, bases):
        return () if P in bases else (P,)


q = Q()
assert types.resolve_bases(()) == ()
assert types.resolve_bases((q,)) == (P,)
assert types.resolve_bases((P,)) == (P,)
assert types.resolve_bases((q, P)) == (P,)
unchanged = (P, Q)
assert types.resolve_bases(unchanged) is unchanged

print("resolve_bases_mro_entries OK")
