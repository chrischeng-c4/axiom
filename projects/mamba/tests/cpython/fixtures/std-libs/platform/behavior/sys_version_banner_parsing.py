# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "sys_version_banner_parsing"
# subject = "platform._sys_version"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
"""platform._sys_version: _sys_version parses an interpreter banner into (name, version, branch, revision, buildno, builddate, compiler), applying the build-date truncation rules and the compiler suffix"""
import platform

CASES = (
    (
        "2.4.3 (#1, Jun 21 2006, 13:54:21) \n[GCC 3.3.4 (pre 3.3.5 20040809)]",
        ("CPython", "2.4.3", "1", "Jun 21 2006 13:54:21",
         "GCC 3.3.4 (pre 3.3.5 20040809)"),
    ),
    (
        "2.4.3 (truncation, date, t) \n[GCC]",
        ("CPython", "2.4.3", "truncation", "date t", "GCC"),
    ),
    (
        "2.4.3 (truncation, date, ) \n[GCC]",
        ("CPython", "2.4.3", "truncation", "date", "GCC"),
    ),
    (
        "2.4.3 (truncation) \n[GCC]",
        ("CPython", "2.4.3", "truncation", "", "GCC"),
    ),
)
for banner, expected in CASES:
    name, version, branch, revision, buildno, builddate, compiler = \
        platform._sys_version(banner)
    got = (name, version, buildno, builddate, compiler)
    assert got == expected, f"parse {banner!r} -> {got!r}"
    assert branch == "", "branch blank without scm tag"
    assert revision == "", "revision blank without scm tag"

print("sys_version_banner_parsing OK")
