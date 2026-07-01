# /// script
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
