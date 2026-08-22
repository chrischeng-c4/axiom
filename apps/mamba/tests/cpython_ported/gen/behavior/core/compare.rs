use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/core/compare/comparison_full_test__test_bytes_uccf0e96.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_full_test__test_bytes_uccf0e96() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_bytes_uccf0e96"
# subject = "cpython.test_compare.ComparisonFullTest.test_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
def assert_total_order(a, b, comp):
    assert (a == b) == (comp == 0)
    assert (b == a) == (comp == 0)
    assert (a != b) == (comp != 0)
    assert (b != a) == (comp != 0)

    assert (a < b) == (comp < 0)
    assert (a <= b) == (comp <= 0)
    assert (a > b) == (comp > 0)
    assert (a >= b) == (comp >= 0)

    assert (b < a) == (comp > 0)
    assert (b <= a) == (comp >= 0)
    assert (b > a) == (comp < 0)
    assert (b >= a) == (comp <= 0)


bs1 = b"a1"
bs2 = b"b2"
assert_total_order(bs1, bs1, 0)
assert_total_order(bs1, bs2, -1)

ba1 = bytearray(b"a1")
ba2 = bytearray(b"b2")
assert_total_order(ba1, ba1, 0)
assert_total_order(ba1, ba2, -1)

assert_total_order(bs1, ba1, 0)
assert_total_order(bs1, ba2, -1)
assert_total_order(ba1, bs1, 0)
assert_total_order(ba1, bs2, -1)

print("ComparisonFullTest::test_bytes: ok")
"###);
    assert_output(&out, r###"ComparisonFullTest::test_bytes: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_full_test__test_comp_classes_different_uc080b1a.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_full_test__test_comp_classes_different_uc080b1a() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_comp_classes_different_uc080b1a"
# subject = "cpython.test_compare.ComparisonFullTest.test_comp_classes_different"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
def expect_type_error(func):
    try:
        func()
    except TypeError as exc:
        assert "not supported" in str(exc)
        return
    raise AssertionError("ordering operation did not raise TypeError")


def assert_eq_subtest(a, b, comp, a_meth, b_meth):
    if "eq" in a_meth or "eq" in b_meth:
        assert (a == b) == (comp == 0)
        assert (b == a) == (comp == 0)
    else:
        assert (a == b) == (a is b)
        assert (b == a) == (a is b)


def assert_ne_subtest(a, b, comp, a_meth, b_meth):
    if (
        "ne" in a_meth
        or "eq" in a_meth
        or "ne" in b_meth
        or "eq" in b_meth
    ):
        assert (a != b) == (comp != 0)
        assert (b != a) == (comp != 0)
    else:
        assert (a != b) == (a is not b)
        assert (b != a) == (a is not b)


def assert_lt_subtest(a, b, comp, a_meth, b_meth):
    if "lt" in a_meth or "gt" in b_meth:
        assert (a < b) == (comp < 0)
        assert (b > a) == (comp < 0)
    else:
        expect_type_error(lambda: a < b)
        expect_type_error(lambda: b > a)


def assert_le_subtest(a, b, comp, a_meth, b_meth):
    if "le" in a_meth or "ge" in b_meth:
        assert (a <= b) == (comp <= 0)
        assert (b >= a) == (comp <= 0)
    else:
        expect_type_error(lambda: a <= b)
        expect_type_error(lambda: b >= a)


def assert_gt_subtest(a, b, comp, a_meth, b_meth):
    if "gt" in a_meth or "lt" in b_meth:
        assert (a > b) == (comp > 0)
        assert (b < a) == (comp > 0)
    else:
        expect_type_error(lambda: a > b)
        expect_type_error(lambda: b < a)


def assert_ge_subtest(a, b, comp, a_meth, b_meth):
    if "ge" in a_meth or "le" in b_meth:
        assert (a >= b) == (comp >= 0)
        assert (b <= a) == (comp >= 0)
    else:
        expect_type_error(lambda: a >= b)
        expect_type_error(lambda: b <= a)


def assert_total_order(a, b, comp, a_meth, b_meth):
    assert_eq_subtest(a, b, comp, a_meth, b_meth)
    assert_ne_subtest(a, b, comp, a_meth, b_meth)
    assert_lt_subtest(a, b, comp, a_meth, b_meth)
    assert_le_subtest(a, b, comp, a_meth, b_meth)
    assert_gt_subtest(a, b, comp, a_meth, b_meth)
    assert_ge_subtest(a, b, comp, a_meth, b_meth)


class CompBase:
    pass


class CompNone(CompBase):
    meth = ()


class CompEq(CompBase):
    meth = ("eq",)

    def __eq__(self, other):
        return self.x == other.x


class CompNe(CompBase):
    meth = ("ne",)

    def __ne__(self, other):
        return self.x != other.x


class CompEqNe(CompBase):
    meth = ("eq", "ne")

    def __eq__(self, other):
        return self.x == other.x

    def __ne__(self, other):
        return self.x != other.x


class CompLt(CompBase):
    meth = ("lt",)

    def __lt__(self, other):
        return self.x < other.x


class CompGt(CompBase):
    meth = ("gt",)

    def __gt__(self, other):
        return self.x > other.x


class CompLtGt(CompBase):
    meth = ("lt", "gt")

    def __lt__(self, other):
        return self.x < other.x

    def __gt__(self, other):
        return self.x > other.x


class CompLe(CompBase):
    meth = ("le",)

    def __le__(self, other):
        return self.x <= other.x


class CompGe(CompBase):
    meth = ("ge",)

    def __ge__(self, other):
        return self.x >= other.x


class CompLeGe(CompBase):
    meth = ("le", "ge")

    def __le__(self, other):
        return self.x <= other.x

    def __ge__(self, other):
        return self.x >= other.x


all_comp_classes = (
    CompNone,
    CompEq,
    CompNe,
    CompEqNe,
    CompLt,
    CompGt,
    CompLtGt,
    CompLe,
    CompGe,
    CompLeGe,
)


for cls_a in all_comp_classes:
    for cls_b in all_comp_classes:
        a1 = cls_a()
        a1.x = 1
        b1 = cls_b()
        b1.x = 1
        b2 = cls_b()
        b2.x = 2

        assert_total_order(a1, b1, 0, cls_a.meth, cls_b.meth)
        assert_total_order(a1, b2, -1, cls_a.meth, cls_b.meth)

print("ComparisonFullTest::test_comp_classes_different: ok")
"###);
    assert_output(&out, r###"ComparisonFullTest::test_comp_classes_different: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_full_test__test_comp_classes_same_uc4aef79.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_full_test__test_comp_classes_same_uc4aef79() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_comp_classes_same_uc4aef79"
# subject = "cpython.test_compare.ComparisonFullTest.test_comp_classes_same"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
def expect_type_error(func):
    try:
        func()
    except TypeError as exc:
        assert "not supported" in str(exc)
        return
    raise AssertionError("ordering operation did not raise TypeError")


def assert_eq_subtest(a, b, comp, a_meth, b_meth):
    if "eq" in a_meth or "eq" in b_meth:
        assert (a == b) == (comp == 0)
        assert (b == a) == (comp == 0)
    else:
        assert (a == b) == (a is b)
        assert (b == a) == (a is b)


def assert_ne_subtest(a, b, comp, a_meth, b_meth):
    if (
        "ne" in a_meth
        or "eq" in a_meth
        or "ne" in b_meth
        or "eq" in b_meth
    ):
        assert (a != b) == (comp != 0)
        assert (b != a) == (comp != 0)
    else:
        assert (a != b) == (a is not b)
        assert (b != a) == (a is not b)


def assert_lt_subtest(a, b, comp, a_meth, b_meth):
    if "lt" in a_meth or "gt" in b_meth:
        assert (a < b) == (comp < 0)
        assert (b > a) == (comp < 0)
    else:
        expect_type_error(lambda: a < b)
        expect_type_error(lambda: b > a)


def assert_le_subtest(a, b, comp, a_meth, b_meth):
    if "le" in a_meth or "ge" in b_meth:
        assert (a <= b) == (comp <= 0)
        assert (b >= a) == (comp <= 0)
    else:
        expect_type_error(lambda: a <= b)
        expect_type_error(lambda: b >= a)


def assert_gt_subtest(a, b, comp, a_meth, b_meth):
    if "gt" in a_meth or "lt" in b_meth:
        assert (a > b) == (comp > 0)
        assert (b < a) == (comp > 0)
    else:
        expect_type_error(lambda: a > b)
        expect_type_error(lambda: b < a)


def assert_ge_subtest(a, b, comp, a_meth, b_meth):
    if "ge" in a_meth or "le" in b_meth:
        assert (a >= b) == (comp >= 0)
        assert (b <= a) == (comp >= 0)
    else:
        expect_type_error(lambda: a >= b)
        expect_type_error(lambda: b <= a)


def assert_total_order(a, b, comp, a_meth, b_meth):
    assert_eq_subtest(a, b, comp, a_meth, b_meth)
    assert_ne_subtest(a, b, comp, a_meth, b_meth)
    assert_lt_subtest(a, b, comp, a_meth, b_meth)
    assert_le_subtest(a, b, comp, a_meth, b_meth)
    assert_gt_subtest(a, b, comp, a_meth, b_meth)
    assert_ge_subtest(a, b, comp, a_meth, b_meth)


class CompBase:
    pass


class CompNone(CompBase):
    meth = ()


class CompEq(CompBase):
    meth = ("eq",)

    def __eq__(self, other):
        return self.x == other.x


class CompNe(CompBase):
    meth = ("ne",)

    def __ne__(self, other):
        return self.x != other.x


class CompEqNe(CompBase):
    meth = ("eq", "ne")

    def __eq__(self, other):
        return self.x == other.x

    def __ne__(self, other):
        return self.x != other.x


class CompLt(CompBase):
    meth = ("lt",)

    def __lt__(self, other):
        return self.x < other.x


class CompGt(CompBase):
    meth = ("gt",)

    def __gt__(self, other):
        return self.x > other.x


class CompLtGt(CompBase):
    meth = ("lt", "gt")

    def __lt__(self, other):
        return self.x < other.x

    def __gt__(self, other):
        return self.x > other.x


class CompLe(CompBase):
    meth = ("le",)

    def __le__(self, other):
        return self.x <= other.x


class CompGe(CompBase):
    meth = ("ge",)

    def __ge__(self, other):
        return self.x >= other.x


class CompLeGe(CompBase):
    meth = ("le", "ge")

    def __le__(self, other):
        return self.x <= other.x

    def __ge__(self, other):
        return self.x >= other.x


def create_sorted_instances(cls, values):
    instances = [cls() for _ in range(len(values))]
    instances.sort(key=id)
    for inst, value in zip(instances, values):
        inst.x = value
    return instances


all_comp_classes = (
    CompNone,
    CompEq,
    CompNe,
    CompEqNe,
    CompLt,
    CompGt,
    CompLtGt,
    CompLe,
    CompGe,
    CompLeGe,
)


for cls in all_comp_classes:
    instances = create_sorted_instances(cls, (1, 2, 1))

    assert_total_order(instances[0], instances[0], 0, cls.meth, cls.meth)
    assert_total_order(instances[0], instances[2], 0, cls.meth, cls.meth)
    assert_total_order(instances[0], instances[1], -1, cls.meth, cls.meth)
    assert_total_order(instances[1], instances[2], 1, cls.meth, cls.meth)

print("ComparisonFullTest::test_comp_classes_same: ok")
"###);
    assert_output(&out, r###"ComparisonFullTest::test_comp_classes_same: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_full_test__test_mappings_uc8f876c.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_full_test__test_mappings_uc8f876c() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_mappings_uc8f876c"
# subject = "cpython.test_compare.ComparisonFullTest.test_mappings"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
def expect_type_error(func):
    try:
        func()
    except TypeError as exc:
        assert "not supported" in str(exc)
        return
    raise AssertionError("ordering operation did not raise TypeError")


def assert_equality_only(a, b, equal):
    assert (a == b) == equal
    assert (b == a) == equal
    assert (a != b) == (not equal)
    assert (b != a) == (not equal)

    expect_type_error(lambda: a < b)
    expect_type_error(lambda: a <= b)
    expect_type_error(lambda: a > b)
    expect_type_error(lambda: a >= b)
    expect_type_error(lambda: b < a)
    expect_type_error(lambda: b <= a)
    expect_type_error(lambda: b > a)
    expect_type_error(lambda: b >= a)


d1 = {1: "a", 2: "b"}
d2 = {2: "b", 3: "c"}
d3 = {3: "c", 2: "b"}

assert_equality_only(d1, d1, True)
assert_equality_only(d1, d2, False)
assert_equality_only(d2, d3, True)

print("ComparisonFullTest::test_mappings: ok")
"###);
    assert_output(&out, r###"ComparisonFullTest::test_mappings: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_full_test__test_numbers_ucdaaeb5.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_full_test__test_numbers_ucdaaeb5() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_numbers_ucdaaeb5"
# subject = "cpython.test_compare.ComparisonFullTest.test_numbers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
from decimal import Decimal
from fractions import Fraction


def expect_type_error(func):
    try:
        func()
    except TypeError as exc:
        assert "not supported" in str(exc)
        return
    raise AssertionError("ordering operation did not raise TypeError")


def assert_equality_only(a, b, equal):
    assert (a == b) == equal
    assert (b == a) == equal
    assert (a != b) == (not equal)
    assert (b != a) == (not equal)

    expect_type_error(lambda: a < b)
    expect_type_error(lambda: a <= b)
    expect_type_error(lambda: a > b)
    expect_type_error(lambda: a >= b)
    expect_type_error(lambda: b < a)
    expect_type_error(lambda: b <= a)
    expect_type_error(lambda: b > a)
    expect_type_error(lambda: b >= a)


def assert_total_order(a, b, comp):
    assert (a == b) == (comp == 0)
    assert (b == a) == (comp == 0)
    assert (a != b) == (comp != 0)
    assert (b != a) == (comp != 0)

    assert (a < b) == (comp < 0)
    assert (a <= b) == (comp <= 0)
    assert (a > b) == (comp > 0)
    assert (a >= b) == (comp >= 0)

    assert (b < a) == (comp > 0)
    assert (b <= a) == (comp >= 0)
    assert (b > a) == (comp < 0)
    assert (b >= a) == (comp <= 0)


i1 = 1001
i2 = 1002
assert_total_order(i1, i1, 0)
assert_total_order(i1, i2, -1)

f1 = 1001.0
f2 = 1001.1
assert_total_order(f1, f1, 0)
assert_total_order(f1, f2, -1)

q1 = Fraction(2002, 2)
q2 = Fraction(2003, 2)
assert_total_order(q1, q1, 0)
assert_total_order(q1, q2, -1)

d1 = Decimal("1001.0")
d2 = Decimal("1001.1")
assert_total_order(d1, d1, 0)
assert_total_order(d1, d2, -1)

c1 = 1001 + 0j
c2 = 1001 + 1j
assert_equality_only(c1, c1, True)
assert_equality_only(c1, c2, False)

for n1, n2 in ((i1, f1), (i1, q1), (i1, d1), (f1, q1), (f1, d1), (q1, d1)):
    assert_total_order(n1, n2, 0)

for n1 in (i1, f1, q1, d1):
    assert_equality_only(n1, c1, True)

print("ComparisonFullTest::test_numbers: ok")
"###);
    assert_output(&out, r###"ComparisonFullTest::test_numbers: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_full_test__test_objects_ucee45b4.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_full_test__test_objects_ucee45b4() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_objects_ucee45b4"
# subject = "cpython.test_compare.ComparisonFullTest.test_objects"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
def expect_type_error(func):
    try:
        func()
    except TypeError as exc:
        assert "not supported" in str(exc)
        return
    raise AssertionError("ordering operation did not raise TypeError")


def assert_equality_only(a, b, equal):
    assert (a == b) == equal
    assert (b == a) == equal
    assert (a != b) == (not equal)
    assert (b != a) == (not equal)

    expect_type_error(lambda: a < b)
    expect_type_error(lambda: a <= b)
    expect_type_error(lambda: a > b)
    expect_type_error(lambda: a >= b)
    expect_type_error(lambda: b < a)
    expect_type_error(lambda: b <= a)
    expect_type_error(lambda: b > a)
    expect_type_error(lambda: b >= a)


a = object()
b = object()
assert_equality_only(a, a, True)
assert_equality_only(a, b, False)

print("ComparisonFullTest::test_objects: ok")
"###);
    assert_output(&out, r###"ComparisonFullTest::test_objects: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_full_test__test_sequences_ucd89ef3.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_full_test__test_sequences_ucd89ef3() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_sequences_ucd89ef3"
# subject = "cpython.test_compare.ComparisonFullTest.test_sequences"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
def expect_type_error(func):
    try:
        func()
    except TypeError as exc:
        assert "not supported" in str(exc)
        return
    raise AssertionError("ordering operation did not raise TypeError")


def assert_equality_only(a, b, equal):
    assert (a == b) == equal
    assert (b == a) == equal
    assert (a != b) == (not equal)
    assert (b != a) == (not equal)

    expect_type_error(lambda: a < b)
    expect_type_error(lambda: a <= b)
    expect_type_error(lambda: a > b)
    expect_type_error(lambda: a >= b)
    expect_type_error(lambda: b < a)
    expect_type_error(lambda: b <= a)
    expect_type_error(lambda: b > a)
    expect_type_error(lambda: b >= a)


def assert_total_order(a, b, comp):
    assert (a == b) == (comp == 0)
    assert (b == a) == (comp == 0)
    assert (a != b) == (comp != 0)
    assert (b != a) == (comp != 0)

    assert (a < b) == (comp < 0)
    assert (a <= b) == (comp <= 0)
    assert (a > b) == (comp > 0)
    assert (a >= b) == (comp >= 0)

    assert (b < a) == (comp > 0)
    assert (b <= a) == (comp >= 0)
    assert (b > a) == (comp < 0)
    assert (b >= a) == (comp <= 0)


l1 = [1, 2]
l2 = [2, 3]
assert_total_order(l1, l1, 0)
assert_total_order(l1, l2, -1)

t1 = (1, 2)
t2 = (2, 3)
assert_total_order(t1, t1, 0)
assert_total_order(t1, t2, -1)

r1 = range(1, 2)
r2 = range(2, 2)
assert_equality_only(r1, r1, True)
assert_equality_only(r1, r2, False)

assert_equality_only(t1, l1, False)
assert_equality_only(l1, r1, False)
assert_equality_only(r1, t1, False)

print("ComparisonFullTest::test_sequences: ok")
"###);
    assert_output(&out, r###"ComparisonFullTest::test_sequences: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_full_test__test_sets_uc4cf128.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_full_test__test_sets_uc4cf128() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_sets_uc4cf128"
# subject = "cpython.test_compare.ComparisonFullTest.test_sets"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
def assert_total_order(a, b, comp):
    assert (a == b) == (comp == 0)
    assert (b == a) == (comp == 0)
    assert (a != b) == (comp != 0)
    assert (b != a) == (comp != 0)

    assert (a < b) == (comp < 0)
    assert (a <= b) == (comp <= 0)
    assert (a > b) == (comp > 0)
    assert (a >= b) == (comp >= 0)

    assert (b < a) == (comp > 0)
    assert (b <= a) == (comp >= 0)
    assert (b > a) == (comp < 0)
    assert (b >= a) == (comp <= 0)


s1 = {1, 2}
s2 = {1, 2, 3}
assert_total_order(s1, s1, 0)
assert_total_order(s1, s2, -1)

f1 = frozenset(s1)
f2 = frozenset(s2)
assert_total_order(f1, f1, 0)
assert_total_order(f1, f2, -1)

assert_total_order(s1, f1, 0)
assert_total_order(s1, f2, -1)
assert_total_order(f1, s1, 0)
assert_total_order(f1, s2, -1)

print("ComparisonFullTest::test_sets: ok")
"###);
    assert_output(&out, r###"ComparisonFullTest::test_sets: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_full_test__test_str_subclass_ucdfb372.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_full_test__test_str_subclass_ucdfb372() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_full_test__test_str_subclass_ucdfb372"
# subject = "cpython.test_compare.ComparisonFullTest.test_str_subclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
def assert_total_order(a, b, comp):
    assert (a == b) == (comp == 0)
    assert (b == a) == (comp == 0)
    assert (a != b) == (comp != 0)
    assert (b != a) == (comp != 0)

    assert (a < b) == (comp < 0)
    assert (a <= b) == (comp <= 0)
    assert (a > b) == (comp > 0)
    assert (a >= b) == (comp >= 0)

    assert (b < a) == (comp > 0)
    assert (b <= a) == (comp >= 0)
    assert (b > a) == (comp < 0)
    assert (b >= a) == (comp <= 0)


class StrSubclass(str):
    pass


s1 = str("a")
s2 = str("b")
c1 = StrSubclass("a")
c2 = StrSubclass("b")
c3 = StrSubclass("b")

assert_total_order(s1, s1, 0)
assert_total_order(s1, s2, -1)
assert_total_order(c1, c1, 0)
assert_total_order(c1, c2, -1)
assert_total_order(c2, c3, 0)

assert_total_order(s1, c2, -1)
assert_total_order(s2, c3, 0)
assert_total_order(c1, s2, -1)
assert_total_order(c2, s2, 0)

print("ComparisonFullTest::test_str_subclass: ok")
"###);
    assert_output(&out, r###"ComparisonFullTest::test_str_subclass: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_simple_test__test_comparisons.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_simple_test__test_comparisons() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_comparisons"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_comparisons"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_compare.py::ComparisonSimpleTest::test_comparisons
"""Auto-ported test: ComparisonSimpleTest::test_comparisons (CPython 3.12 oracle)."""


class Empty:
    def __repr__(self):
        return "<Empty>"


class Cmp:
    def __init__(self, arg):
        self.arg = arg

    def __repr__(self):
        return f"<Cmp {self.arg}>"

    def __eq__(self, other):
        return self.arg == other


set1 = [2, 2.0, 2, 2 + 0j, Cmp(2.0)]
set2 = [[1], (3,), None, Empty()]
candidates = set1 + set2

for a in candidates:
    for b in candidates:
        if ((a in set1) and (b in set1)) or a is b:
            assert a == b, (a, b)
        else:
            assert a != b, (a, b)

print("ComparisonSimpleTest::test_comparisons: ok")
"###);
    assert_output(&out, r###"ComparisonSimpleTest::test_comparisons: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_simple_test__test_id_comparisons_uc3dfcaa.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_simple_test__test_id_comparisons_uc3dfcaa() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_id_comparisons_uc3dfcaa"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_id_comparisons"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
class Empty:
    def __repr__(self):
        return "<Empty>"


items = []
for i in range(10):
    items.insert(len(items) // 2, Empty())

for a in items:
    for b in items:
        assert (a == b) == (a is b), "a=%r, b=%r" % (a, b)

print("ComparisonSimpleTest::test_id_comparisons: ok")
"###);
    assert_output(&out, r###"ComparisonSimpleTest::test_id_comparisons: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_simple_test__test_issue_1393_uc8f23ca.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_simple_test__test_issue_1393_uc8f23ca() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_issue_1393_uc8f23ca"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_issue_1393"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
class AlwaysEq:
    def __eq__(self, other):
        return True

    def __ne__(self, other):
        return False


always_eq = AlwaysEq()

x = lambda: None
assert x == always_eq
assert always_eq == x

y = object()
assert y == always_eq
assert always_eq == y

print("ComparisonSimpleTest::test_issue_1393: ok")
"###);
    assert_output(&out, r###"ComparisonSimpleTest::test_issue_1393: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_simple_test__test_ne_defaults_to_not_eq_uc0267c8.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_simple_test__test_ne_defaults_to_not_eq_uc0267c8() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_ne_defaults_to_not_eq_uc0267c8"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_ne_defaults_to_not_eq"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
class Cmp:
    def __init__(self, arg):
        self.arg = arg

    def __eq__(self, other):
        return self.arg == other.arg


a = Cmp(1)
b = Cmp(1)
c = Cmp(2)
assert (a == b) is True
assert (a != b) is False
assert (a != c) is True

print("ComparisonSimpleTest::test_ne_defaults_to_not_eq: ok")
"###);
    assert_output(&out, r###"ComparisonSimpleTest::test_ne_defaults_to_not_eq: ok
"###);
}

/// Ported from `tests/cpython/behavior/core/compare/comparison_simple_test__test_other_delegation_uc2d0ffb.py`.
#[test]
fn test_gen_behavior_core_compare_comparison_simple_test__test_other_delegation_uc2d0ffb() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "compare"
# dimension = "behavior"
# case = "comparison_simple_test__test_other_delegation_uc2d0ffb"
# subject = "cpython.test_compare.ComparisonSimpleTest.test_other_delegation"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_compare.py"
# status = "filled"
# ///
def boom_ne(*args):
    raise AssertionError("__ne__ called")


def boom_eq(*args):
    raise AssertionError("__eq__ called")


def boom_lt(*args):
    raise AssertionError("__lt__ called")


def boom_le(*args):
    raise AssertionError("__le__ called")


def boom_gt(*args):
    raise AssertionError("__gt__ called")


def boom_ge(*args):
    raise AssertionError("__ge__ called")


def expect_type_error(func):
    try:
        func()
    except TypeError:
        return
    raise AssertionError("missing rich comparison method did not raise TypeError")


class MissingEq:
    __ne__ = boom_ne
    __lt__ = boom_lt
    __le__ = boom_le
    __gt__ = boom_gt
    __ge__ = boom_ge


assert (MissingEq() == object()) is False


class MissingLt:
    __ne__ = boom_ne
    __eq__ = boom_eq
    __le__ = boom_le
    __gt__ = boom_gt
    __ge__ = boom_ge


expect_type_error(lambda: MissingLt() < object())


class MissingLe:
    __ne__ = boom_ne
    __eq__ = boom_eq
    __lt__ = boom_lt
    __gt__ = boom_gt
    __ge__ = boom_ge


expect_type_error(lambda: MissingLe() <= object())


class MissingGt:
    __ne__ = boom_ne
    __eq__ = boom_eq
    __lt__ = boom_lt
    __le__ = boom_le
    __ge__ = boom_ge


expect_type_error(lambda: MissingGt() > object())


class MissingGe:
    __ne__ = boom_ne
    __eq__ = boom_eq
    __lt__ = boom_lt
    __le__ = boom_le
    __gt__ = boom_gt


expect_type_error(lambda: MissingGe() >= object())

print("ComparisonSimpleTest::test_other_delegation: ok")
"###);
    assert_output(&out, r###"ComparisonSimpleTest::test_other_delegation: ok
"###);
}
