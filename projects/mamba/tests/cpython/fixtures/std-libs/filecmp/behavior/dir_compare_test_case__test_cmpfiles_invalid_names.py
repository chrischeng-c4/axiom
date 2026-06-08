# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "behavior"
# case = "dir_compare_test_case__test_cmpfiles_invalid_names"
# subject = "cpython.test_filecmp.DirCompareTestCase.test_cmpfiles_invalid_names"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_filecmp.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_filecmp.py::DirCompareTestCase::test_cmpfiles_invalid_names
"""Auto-ported test: DirCompareTestCase::test_cmpfiles_invalid_names."""


import filecmp
import os
import tempfile


with tempfile.TemporaryDirectory() as root:
    left = os.path.join(root, "dir")
    right_same = os.path.join(root, "dir-same")
    right_diff = os.path.join(root, "dir-diff")

    for directory in (left, right_same, right_diff):
        os.mkdir(directory)
        with open(os.path.join(directory, "file"), "w", encoding="utf-8") as output:
            output.write("Contents of file go here.\n")

    with open(os.path.join(right_diff, "file2"), "w", encoding="utf-8") as output:
        output.write("An extra file.\n")

    for name, desc in [
        ("\x00", "NUL bytes filename"),
        (__file__ + "\x00", "filename with embedded NUL bytes"),
        ("\ud834\udd1e.py", "surrogate codes (MUSICAL SYMBOL G CLEF)"),
        ("a" * 1_000_000, "very long filename"),
    ]:
        for other_dir in [left, right_same, right_diff]:
            result = filecmp.cmpfiles(left, other_dir, [name])
            assert result == ([], [], [name]), (desc, other_dir, result)

print("DirCompareTestCase::test_cmpfiles_invalid_names: ok")
