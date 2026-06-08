# Operational AssertionPass seed for the `operator` stdlib module's
# in-place / truth / identity entry-point surface — the matching
# subset that the existing `test_operator.py`,
# `test_operator_arith_compare_ops.py`,
# `test_operator_arith_extras_ops.py`, and
# `test_operator_bitwise_seq_ops.py` do NOT already exercise. Those
# files cover the non-in-place arithmetic / bitwise / comparison /
# sequence-protocol surface (`add`, `sub`, `mul`, `truediv`,
# `floordiv`, `mod`, `pow`, `and_`, `or_`, `xor`, `lshift`,
# `rshift`, `concat`, `eq`/`ne`/`lt`/`le`/`gt`/`ge`, `contains`,
# `countOf`, `indexOf`). This seed fills the orthogonal IN-PLACE
# family (`iadd`, `isub`, `imul`, `ipow`, `imod`, `ifloordiv`,
# `iand`, `ior`, `ixor`, `ilshift`, `irshift`, `iconcat`) plus the
# `truth` / `not_` / `is_` / `is_not` identity-and-truth callables
# and the `matmul` module-attribute discipline.
#
# Surface (the matching subset between mamba and CPython):
#   • operator.iadd / isub / imul / ipow / imod / ifloordiv —
#     return the same numeric result as their non-in-place
#     partners for `int` operands;
#   • operator.iand / ior / ixor / ilshift / irshift — return the
#     same bitwise result as their non-in-place partners;
#   • operator.iconcat — same sequence-concat result as `concat`;
#   • in-place fn (int operand) == non-in-place fn (int operand)
#     for every pair above;
#   • operator.truth(x) is True for truthy x, False for falsy x;
#   • operator.not_(x) is the bool-negation of operator.truth(x);
#   • operator.is_(a, b) is `a is b`;
#   • operator.is_not(a, b) is `a is not b`;
#   • operator.matmul is callable (binary @ entry point);
#   • module attribute discipline — operator.__name__ == 'operator'.
import operator
_ledger: list[int] = []

# operator.iadd — int + int
assert operator.iadd(0, 0) == 0; _ledger.append(1)
assert operator.iadd(5, 3) == 8; _ledger.append(1)
assert operator.iadd(-5, 3) == -2; _ledger.append(1)
assert operator.iadd(100, 200) == 300; _ledger.append(1)
assert operator.iadd(5, 3) == operator.add(5, 3); _ledger.append(1)

# operator.isub — int - int
assert operator.isub(10, 4) == 6; _ledger.append(1)
assert operator.isub(0, 5) == -5; _ledger.append(1)
assert operator.isub(-5, -3) == -2; _ledger.append(1)
assert operator.isub(10, 4) == operator.sub(10, 4); _ledger.append(1)

# operator.imul — int * int
assert operator.imul(3, 5) == 15; _ledger.append(1)
assert operator.imul(-5, 3) == -15; _ledger.append(1)
assert operator.imul(0, 100) == 0; _ledger.append(1)
assert operator.imul(3, 5) == operator.mul(3, 5); _ledger.append(1)

# operator.ipow — int ** int
assert operator.ipow(2, 10) == 1024; _ledger.append(1)
assert operator.ipow(3, 4) == 81; _ledger.append(1)
assert operator.ipow(2, 0) == 1; _ledger.append(1)
assert operator.ipow(2, 10) == operator.pow(2, 10); _ledger.append(1)

# operator.imod — int % int
assert operator.imod(10, 3) == 1; _ledger.append(1)
assert operator.imod(20, 7) == 6; _ledger.append(1)
assert operator.imod(-7, 3) == 2; _ledger.append(1)
assert operator.imod(10, 3) == operator.mod(10, 3); _ledger.append(1)

# operator.ifloordiv — int // int
assert operator.ifloordiv(10, 3) == 3; _ledger.append(1)
assert operator.ifloordiv(20, 7) == 2; _ledger.append(1)
assert operator.ifloordiv(-7, 3) == -3; _ledger.append(1)
assert operator.ifloordiv(10, 3) == operator.floordiv(10, 3); _ledger.append(1)

# operator.iand — int & int
assert operator.iand(0b1100, 0b1010) == 0b1000; _ledger.append(1)
assert operator.iand(0xff, 0x0f) == 0x0f; _ledger.append(1)
assert operator.iand(0, 0xff) == 0; _ledger.append(1)
assert operator.iand(0b1100, 0b1010) == operator.and_(0b1100, 0b1010); _ledger.append(1)

# operator.ior — int | int
assert operator.ior(0b1100, 0b1010) == 0b1110; _ledger.append(1)
assert operator.ior(0x0f, 0xf0) == 0xff; _ledger.append(1)
assert operator.ior(0, 0xff) == 0xff; _ledger.append(1)
assert operator.ior(0b1100, 0b1010) == operator.or_(0b1100, 0b1010); _ledger.append(1)

# operator.ixor — int ^ int
assert operator.ixor(0b1100, 0b1010) == 0b0110; _ledger.append(1)
assert operator.ixor(0xff, 0xff) == 0; _ledger.append(1)
assert operator.ixor(0, 0xff) == 0xff; _ledger.append(1)
assert operator.ixor(0b1100, 0b1010) == operator.xor(0b1100, 0b1010); _ledger.append(1)

# operator.ilshift — int << int
assert operator.ilshift(1, 4) == 16; _ledger.append(1)
assert operator.ilshift(5, 1) == 10; _ledger.append(1)
assert operator.ilshift(0, 10) == 0; _ledger.append(1)
assert operator.ilshift(1, 4) == operator.lshift(1, 4); _ledger.append(1)

# operator.irshift — int >> int
assert operator.irshift(64, 2) == 16; _ledger.append(1)
assert operator.irshift(5, 1) == 2; _ledger.append(1)
assert operator.irshift(0, 10) == 0; _ledger.append(1)
assert operator.irshift(64, 2) == operator.rshift(64, 2); _ledger.append(1)

# operator.iconcat — list ++ list
assert operator.iconcat([], []) == []; _ledger.append(1)
assert operator.iconcat([1], [2]) == [1, 2]; _ledger.append(1)
assert operator.iconcat([1, 2], [3, 4]) == [1, 2, 3, 4]; _ledger.append(1)
assert operator.iconcat([1, 2], [3, 4]) == operator.concat([1, 2], [3, 4]); _ledger.append(1)

# operator.iconcat — str ++ str
assert operator.iconcat("", "") == ""; _ledger.append(1)
assert operator.iconcat("a", "b") == "ab"; _ledger.append(1)
assert operator.iconcat("hello", " world") == "hello world"; _ledger.append(1)

# operator.iconcat — tuple ++ tuple
assert operator.iconcat((), ()) == (); _ledger.append(1)
assert operator.iconcat((1,), (2,)) == (1, 2); _ledger.append(1)
assert operator.iconcat((1, 2), (3, 4)) == (1, 2, 3, 4); _ledger.append(1)

# operator.truth — falsy
assert operator.truth(0) == False; _ledger.append(1)
assert operator.truth(0.0) == False; _ledger.append(1)
assert operator.truth("") == False; _ledger.append(1)
assert operator.truth([]) == False; _ledger.append(1)
assert operator.truth({}) == False; _ledger.append(1)
assert operator.truth(()) == False; _ledger.append(1)
assert operator.truth(None) == False; _ledger.append(1)

# operator.truth — truthy
assert operator.truth(1) == True; _ledger.append(1)
assert operator.truth(-1) == True; _ledger.append(1)
assert operator.truth(0.1) == True; _ledger.append(1)
assert operator.truth("a") == True; _ledger.append(1)
assert operator.truth([0]) == True; _ledger.append(1)
assert operator.truth({0: 0}) == True; _ledger.append(1)
assert operator.truth((0,)) == True; _ledger.append(1)

# operator.not_ — bool negation of truth
assert operator.not_(0) == True; _ledger.append(1)
assert operator.not_(1) == False; _ledger.append(1)
assert operator.not_("") == True; _ledger.append(1)
assert operator.not_("a") == False; _ledger.append(1)
assert operator.not_([]) == True; _ledger.append(1)
assert operator.not_([1]) == False; _ledger.append(1)
assert operator.not_(None) == True; _ledger.append(1)

# operator.truth and not_ are complements
assert operator.not_(0) != operator.truth(0); _ledger.append(1)
assert operator.not_(1) != operator.truth(1); _ledger.append(1)
assert operator.not_([]) != operator.truth([]); _ledger.append(1)
assert operator.not_([1, 2]) != operator.truth([1, 2]); _ledger.append(1)

# operator.is_ — identity
assert operator.is_(None, None) == True; _ledger.append(1)
assert operator.is_(True, True) == True; _ledger.append(1)
assert operator.is_(False, False) == True; _ledger.append(1)
_x = [1, 2]
assert operator.is_(_x, _x) == True; _ledger.append(1)

# operator.is_not — inverse of is_
assert operator.is_not(None, 1) == True; _ledger.append(1)
assert operator.is_not(1, 1.0) == True; _ledger.append(1)
assert operator.is_not(None, None) == False; _ledger.append(1)
_y = [1, 2]
assert operator.is_not(_x, _y) == True; _ledger.append(1)

# operator.matmul — module attribute discipline
assert callable(operator.matmul); _ledger.append(1)
assert hasattr(operator, "matmul"); _ledger.append(1)

# In-place family — module attribute discipline
for _name in ("iadd", "isub", "imul", "ipow", "imod", "ifloordiv",
              "iand", "ior", "ixor", "ilshift", "irshift", "iconcat"):
    assert hasattr(operator, _name); _ledger.append(1)
    assert callable(getattr(operator, _name)); _ledger.append(1)

# Truth / identity family — module attribute discipline
for _name in ("truth", "not_", "is_", "is_not"):
    assert hasattr(operator, _name); _ledger.append(1)
    assert callable(getattr(operator, _name)); _ledger.append(1)

# Module name discipline
assert operator.__name__ == "operator"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_operator_inplace_truth_caller_ops {sum(_ledger)} asserts")
