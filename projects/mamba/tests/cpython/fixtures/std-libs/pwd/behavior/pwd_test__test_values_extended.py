# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pwd"
# dimension = "behavior"
# case = "pwd_test__test_values_extended"
# subject = "cpython.test_pwd.PwdTest.test_values_extended"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pwd.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
"""Auto-ported test: PwdTest::test_values_extended (CPython 3.12 oracle)."""

import pwd


entries = pwd.getpwall()

if len(entries) > 1000:
    print("PwdTest::test_values_extended skipped: passwd file is huge")
else:
    entries_by_name = {}
    entries_by_uid = {}

    for entry in entries:
        entries_by_name.setdefault(entry.pw_name, []).append(entry)
        entries_by_uid.setdefault(entry.pw_uid, []).append(entry)

    for entry in entries:
        if not entry[0] or entry[0] == "+":
            continue
        assert pwd.getpwnam(entry.pw_name) in entries_by_name[entry.pw_name]
        assert pwd.getpwuid(entry.pw_uid) in entries_by_uid[entry.pw_uid]

    print("PwdTest::test_values_extended: ok")
