# Operational AssertionPass seed for the `copy` stdlib module.
# Surface: deepcopy isolates nested mutation, copy of flat list does
# not alias the outer list but shares inner refs.
# Companion to stub/test_copy.py — vendored unittest seed.
import copy
_ledger: list[int] = []

xs = [[1, 2], [3, 4]]
ys = copy.deepcopy(xs)
ys[0].append(99)
assert xs == [[1, 2], [3, 4]]; _ledger.append(1)
assert ys == [[1, 2, 99], [3, 4]]; _ledger.append(1)

flat = [1, 2, 3]
shallow = copy.copy(flat)
shallow.append(4)
assert flat == [1, 2, 3]; _ledger.append(1)
assert shallow == [1, 2, 3, 4]; _ledger.append(1)

nested = {"a": [1, 2], "b": [3, 4]}
deep = copy.deepcopy(nested)
deep["a"].append(99)
assert nested["a"] == [1, 2]; _ledger.append(1)
assert deep["a"] == [1, 2, 99]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_copy_ops {sum(_ledger)} asserts")
