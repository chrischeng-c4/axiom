# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "clinic"
# dimension = "behavior"
# case = "clinic_group_permuter_test__test_range"
# subject = "cpython.test_clinic.ClinicGroupPermuterTest.test_range"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_clinic.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_clinic.py::ClinicGroupPermuterTest::test_range
"""Auto-ported test: ClinicGroupPermuterTest::test_range."""


try:
    from test import test_tools

    test_tools.skip_if_missing("clinic")
except BaseException as exc:
    print(f"ClinicGroupPermuterTest::test_range: skipped; {exc}")
    raise SystemExit(0)

with test_tools.imports_under_tool("clinic"):
    import clinic


computed = clinic.permute_optional_groups([["start"]], ["stop"], [["step"]])
expected = (
    ("stop",),
    ("start", "stop"),
    ("start", "stop", "step"),
)

assert computed == expected, computed

print("ClinicGroupPermuterTest::test_range: ok")
