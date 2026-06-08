# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "surface"
# case = "checkfuncname_is_callable"
# subject = "bdb.checkfuncname"
# kind = "mechanical"
# xfail = "mamba bdb stub: checkfuncname absent (#1261; stdlib-stub-audit 2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.checkfuncname: checkfuncname_is_callable (surface)."""
import bdb

assert callable(bdb.checkfuncname)
print("checkfuncname_is_callable OK")
