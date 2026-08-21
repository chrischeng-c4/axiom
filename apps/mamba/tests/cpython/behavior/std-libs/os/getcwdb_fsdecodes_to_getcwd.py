# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "getcwdb_fsdecodes_to_getcwd"
# subject = "os.getcwdb"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.getcwdb: os.getcwdb returns bytes that os.fsdecode back to exactly os.getcwd()"""
import os

cwdb = os.getcwdb()
assert isinstance(cwdb, bytes), f"getcwdb type = {type(cwdb)!r}"
assert os.fsdecode(cwdb) == os.getcwd(), "getcwdb fsdecodes to getcwd"
print("getcwdb_fsdecodes_to_getcwd OK")
