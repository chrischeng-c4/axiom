# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "behavior"
# case = "namedtuple_rename_replaces_invalid"
# subject = "collections.namedtuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: rename=True replaces each invalid/duplicate/keyword/leading-underscore field with _<index>"""
from collections import namedtuple

assert namedtuple("NT", ["abc", "def"], rename=True)._fields == ("abc", "_1"), "keyword field renamed"
assert namedtuple("NT", ["8efg", "9ghi"], rename=True)._fields == ("_0", "_1"), "leading-digit fields renamed"
assert namedtuple("NT", ["abc", "efg", "efg", "ghi"], rename=True)._fields == (
    "abc", "efg", "_2", "ghi",
), "duplicate field renamed"

print("namedtuple_rename_replaces_invalid OK")
