# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "pure_path_parsing"
# subject = "pathlib.PurePath"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
"""pathlib.PurePath: PurePath parsing: anchor/root/drive on absolute vs relative, multi-arg and PurePath-instance constructors, the parent chain bottoming out at '.'/'/', the parents sequence (index/negative/slice/iterate/immutable), and name/stem/suffix dot edge cases"""
import pathlib

P = pathlib.PurePath

# Anchor / root / drive on POSIX-style paths.
assert P("").anchor == "", "empty anchor"
assert P("a/b").anchor == "", "relative anchor"
assert P("/a/b").anchor == "/", f"absolute anchor = {P('/a/b').anchor!r}"
assert P("/a/b").root == "/", "absolute root"
assert P("a/b").root == "", "relative root"
assert P("/a/b").drive == "", "posix has no drive"

# Constructor accepts multiple args and PurePath instances.
assert P("a", "b", "c") == P("a/b/c"), "multi-arg join"
assert P(P("a"), "b") == P("a/b"), "PurePath + str"
assert P(P("a"), P("b"), P("c")) == P("a/b/c"), "PurePath chain"
assert P(P("a")) == P("a"), "PurePath copy"

# Parent chain bottoms out at '.' (relative) or '/' (absolute).
_rel = P("a/b/c")
assert _rel.parent == P("a/b"), "rel parent"
assert _rel.parent.parent.parent == P("."), "rel parent floor is ."
assert _rel.parent.parent.parent.parent == P("."), "rel parent floor sticks"
_abs = P("/a/b/c")
assert _abs.parent.parent.parent == P("/"), "abs parent floor is /"
assert _abs.parent.parent.parent.parent == P("/"), "abs parent floor sticks"

# parents is an indexable sequence with negative indexing and slicing.
_par = P("a/b/c").parents
assert len(_par) == 3, f"parents len = {len(_par)!r}"
assert _par[0] == P("a/b"), "parents[0]"
assert _par[2] == P("."), "parents[-1] is ."
assert _par[-1] == P("."), "negative index"
assert _par[:2] == (P("a/b"), P("a")), "slice"
assert _par[::-1] == (P("."), P("a"), P("a/b")), "reverse slice"
assert list(_par) == [P("a/b"), P("a"), P(".")], "iterate"

# Out-of-range index raises IndexError; the sequence is immutable.
try:
    _par[3]
    raise AssertionError("parents[3] should raise IndexError")
except IndexError:
    pass
try:
    _par[0] = P("x")  # type: ignore[index]
    raise AssertionError("parents assignment should raise TypeError")
except TypeError:
    pass

# name/stem/suffix edge cases around dots.
assert P("..").stem == "..", "double-dot stem"
assert P("a/.hgrc").stem == ".hgrc", "dotfile stem"
assert P("a/.hg.rc").suffix == ".rc", "dotfile suffix"
assert P("a/Dot ending.").suffix == "", "trailing dot has no suffix"
assert P("/a/b/.").name == "b", "trailing /. ignored for name"
print("pure_path_parsing OK")
