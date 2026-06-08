# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "isrecursive_isreadable_predicates"
# subject = "pprint.isrecursive"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.isrecursive: isrecursive reports reference cycles and isreadable reports eval-back-ability: True for acyclic literals, False for cyclic structures and non-literal reprs (functions/modules/object()); breaking a cycle restores readability"""
import pprint

pp = pprint.PrettyPrinter()

# Acyclic, eval-able values: not recursive, and readable.
for safe in (2, 2.0, 2j, "abc", [3], (2, 2), {3: 3},
             b"def", bytearray(b"ghi"), True, False, None, ...):
    assert not pprint.isrecursive(safe)
    assert pprint.isreadable(safe)
    assert not pp.isrecursive(safe)
    assert pp.isreadable(safe)

# A self-referential list is recursive and therefore not readable.
cyclic: list = [1, 2]
cyclic.append(cyclic)
assert pprint.isrecursive(cyclic)
assert not pprint.isreadable(cyclic)

# A self-referential dict, and a tuple wrapping it, are recursive too.
d: dict = {}
d[0] = d[1] = d
for icky in (d, (d, d)):
    assert pprint.isrecursive(icky)
    assert not pprint.isreadable(icky)

# Breaking the cycle restores readability.
d.clear()
assert not pprint.isrecursive(d)
assert pprint.isreadable(d)

# Objects with non-literal reprs are readable=False but not recursive.
for unreadable in (object(), int, pprint, pprint.isrecursive):
    assert not pprint.isrecursive(unreadable)
    assert not pprint.isreadable(unreadable)
print("isrecursive_isreadable_predicates OK")
