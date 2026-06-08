# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "errors"
# case = "verify_unique_rejects_duplicate"
# subject = "enum.verify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.verify: @verify(UNIQUE) on an IntEnum with two members sharing a value raises ValueError (the decorator form of @unique)"""
import enum


# @verify(UNIQUE) is the decorator form of @unique: aliases are rejected.
_raised = False
try:
    @enum.verify(enum.UNIQUE)
    class Dup(enum.IntEnum):
        X = 1
        Y = 1
except ValueError:
    _raised = True
assert _raised, "@verify(UNIQUE) on a class with a duplicate value must raise"

print("verify_unique_rejects_duplicate OK")
