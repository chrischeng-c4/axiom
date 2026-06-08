# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_flavour_semantics"
# subject = "pathlib.PurePosixPath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePosixPath: POSIX-flavour rules: case sensitivity, single/double/triple-slash root handling, is_absolute, absolute-component reset on join, is_reserved always False, and coercion of a PureWindowsPath string form"""
import pathlib

P = pathlib.PurePosixPath
PureWindowsPath = pathlib.PureWindowsPath

# POSIX paths are case-sensitive.
assert P("a/b") != P("A/b"), "case-sensitive inequality"

# A single leading slash is one root; a lone double slash is a distinct
# "//" root (POSIX-implementation-defined), but three-or-more collapse to one.
assert P("/a").root == "/", "single-slash root"
assert P("//a").root == "//", f"double-slash root = {P('//a').root!r}"
assert P("///a").root == "/", "triple-slash collapses"
assert P("/a") == P("///a"), "/a == ///a"
assert P("/a") != P("//a"), "/a != //a (distinct roots)"

# is_absolute: any leading slash (including //) is absolute.
assert not P().is_absolute(), "empty is relative"
assert not P("a/b").is_absolute(), "relative path"
assert P("/a/b").is_absolute(), "leading slash is absolute"
assert P("//a/b").is_absolute(), "double-slash is absolute"

# Joining an absolute component discards everything before it.
assert P("/a") / "//c" == P("//c"), "// component resets"
assert P("//a") / "/c" == P("/c"), "/ component resets"
assert P("//a") / "b" == P("//a/b"), "double-slash root preserved on join"
assert P("/a").joinpath("//c") == P("//c"), "joinpath absolute reset"

# is_reserved is always False on POSIX, even for Windows-reserved-looking names.
assert P("").is_reserved() is False, "empty not reserved"
assert P("/dev/con/PRN/NUL").is_reserved() is False, "con/prn/nul not reserved"

# Constructing a POSIX path from a Windows pure path coerces its string form;
# a 'c:' becomes an ordinary path component (no drive on POSIX).
assert P("c:", "a", "b") == P(PureWindowsPath("c:\\a\\b")), "windows coercion"
print("posix_flavour_semantics OK")
