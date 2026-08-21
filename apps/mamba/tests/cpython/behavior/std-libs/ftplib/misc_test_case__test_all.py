# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ftplib"
# dimension = "behavior"
# case = "misc_test_case__test_all"
# subject = "cpython.test_ftplib.MiscTestCase.test__all__"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ftplib.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_ftplib.py::MiscTestCase::test__all__
"""Auto-ported test: MiscTestCase::test__all__ (CPython 3.12 oracle)."""


import ftplib


not_exported = {
    "MSG_OOB",
    "FTP_PORT",
    "MAXLINE",
    "CRLF",
    "B_CRLF",
    "Error",
    "parse150",
    "parse227",
    "parse229",
    "parse257",
    "print_line",
    "ftpcp",
    "test",
}

for name in not_exported:
    assert name not in ftplib.__all__, name

for name in ("FTP", "FTP_TLS", "all_errors", "error_perm", "error_proto", "error_reply", "error_temp"):
    assert name in ftplib.__all__, name
    assert hasattr(ftplib, name), name

print("MiscTestCase::test__all__: ok")
