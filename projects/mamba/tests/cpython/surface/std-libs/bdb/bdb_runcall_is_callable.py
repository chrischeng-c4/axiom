# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "bdb"
# dimension = "surface"
# case = "bdb_runcall_is_callable"
# subject = "bdb.Bdb.runcall"
# kind = "mechanical"
# xfail = "mamba bdb stub: Bdb has no runcall method (#1261; stdlib-stub-audit 2026-05-26)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""bdb.Bdb.runcall: bdb_runcall_is_callable (surface)."""
import bdb

assert callable(bdb.Bdb.runcall)
print("bdb_runcall_is_callable OK")
