# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "api_legacy_transaction_control_is_present"
# subject = "sqlite3.LEGACY_TRANSACTION_CONTROL"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""sqlite3.LEGACY_TRANSACTION_CONTROL: api_legacy_transaction_control_is_present (surface)."""
import sqlite3

assert hasattr(sqlite3, "LEGACY_TRANSACTION_CONTROL")
print("api_legacy_transaction_control_is_present OK")
