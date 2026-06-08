# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "copy"
# dimension = "behavior"
# case = "shallow_outer_mutation_isolated"
# subject = "copy.copy"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_copy.py"
# status = "filled"
# ///
"""copy.copy: appending to a shallow-copied list does not change the original's length, but mutating a shared inner list is visible in both"""
import copy

original = [[1, 2], [3, 4]]
shallow = copy.copy(original)

# Appending to the shallow copy's outer list leaves the original's length alone.
shallow.append([5, 6])
assert len(original) == 2, f"original len unchanged = {len(original)!r}"
assert len(shallow) == 3, "shallow outer extended"

# But mutating a shared inner list is visible through the original.
shallow[0].append(99)
assert original[0] == [1, 2, 99], f"inner mutation visible in original: {original[0]!r}"

print("shallow_outer_mutation_isolated OK")
