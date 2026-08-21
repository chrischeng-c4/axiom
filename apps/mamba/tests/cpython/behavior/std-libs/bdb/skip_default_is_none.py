# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "behavior"
# case = "skip_default_is_none"
# subject = "bdb.Bdb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb: a freshly constructed Bdb has skip == None (no skip patterns configured)"""
import bdb

_d = bdb.Bdb()
assert _d.skip is None, f"default skip = {_d.skip!r}"

print("skip_default_is_none OK")
