# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "future_stmt"
# dimension = "behavior"
# case = "future_test__test_attributes"
# subject = "cpython.test.test_future_stmt.test_future_flags.FutureTest.test_attributes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_future_stmt/test_future_flags.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_future_flags.py::FutureTest::test_attributes
"""Auto-ported test: FutureTest::test_attributes (CPython 3.12 oracle)."""


import __future__


GOOD_SERIALS = ("alpha", "beta", "candidate", "final")

features = __future__.all_feature_names


def check_release_tuple(value, name):
    assert isinstance(value, tuple), f"{name} isn't tuple"
    assert len(value) == 5, f"{name} isn't 5-tuple"
    major, minor, micro, level, serial = value
    assert isinstance(major, int), f"{name} major isn't int"
    assert isinstance(minor, int), f"{name} minor isn't int"
    assert isinstance(micro, int), f"{name} micro isn't int"
    assert isinstance(level, str), f"{name} level isn't string"
    assert level in GOOD_SERIALS, f"{name} level string has unknown value"
    assert isinstance(serial, int), f"{name} serial isn't int"


for feature in features:
    value = getattr(__future__, feature)

    optional = value.getOptionalRelease()
    mandatory = value.getMandatoryRelease()

    check_release_tuple(optional, "optional")
    if mandatory is not None:
        check_release_tuple(mandatory, "mandatory")
        assert optional < mandatory, "optional not less than mandatory, and mandatory not None"

    assert hasattr(value, "compiler_flag"), "feature is missing a .compiler_flag attr"
    compile("", "<test>", "exec", value.compiler_flag)
    assert isinstance(getattr(value, "compiler_flag"), int), ".compiler_flag isn't int"

print("FutureTest::test_attributes: ok")
