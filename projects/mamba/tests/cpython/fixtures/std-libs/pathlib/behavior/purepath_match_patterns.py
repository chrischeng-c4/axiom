# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "purepath_match_patterns"
# subject = "pathlib.PurePosixPath.match"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePosixPath.match: PurePath.match glob semantics: tail-matching of relative patterns, wildcards, leading-slash anchoring, '**' segment spanning, the case_sensitive flag, and empty-path vs '*'/'**' behavior"""
import pathlib

P = pathlib.PurePosixPath

# A relative pattern matches against the tail of the path.
assert P("a/b.py").match("b.py"), "tail match"
assert P("/a/b.py").match("b.py"), "tail match on absolute"
assert not P("a.py").match("b.py"), "name mismatch"
assert not P("b.py/c").match("b.py"), "must match final component"

# Wildcards.
assert P("a/b.py").match("*.py"), "* suffix"
assert not P("b.pyc").match("*.py"), "* must be exact suffix"
assert P("ab/c.py").match("a*/*.py"), "multi-segment wildcard"
assert not P("ab/c.py/d").match("a*/*.py"), "trailing component breaks match"

# A leading slash anchors the pattern to the path root.
assert P("/b.py").match("/*.py"), "anchored match"
assert not P("a/b.py").match("/*.py"), "anchored pattern rejects relative path"
assert P("/a/b.py").match("/a/*.py"), "anchored multi-segment"
assert not P("/a/b/c.py").match("/a/*.py"), "* spans one segment only"

# '**' matches zero-or-more segments only when explicitly anchored.
assert not P("/a/b/c.py").match("/**/*.py"), "leading /** still needs a segment"
assert P("/a/b/c.py").match("/a/**/*.py"), "/** spans segments"

# case_sensitive overrides the flavour default.
assert not P("A.py").match("a.PY", case_sensitive=True), "case-sensitive mismatch"
assert P("A.py").match("a.PY", case_sensitive=False), "case-insensitive match"
assert P("/a/b/c.py").match("/A/*/*.Py", case_sensitive=False), "ci anchored"

# Empty path matches '**' (zero segments) but not '*'.
assert not P().match("*"), "empty path vs *"
assert P().match("**"), "empty path vs **"
assert not P().match("**/*"), "empty path vs **/*"
print("purepath_match_patterns OK")
