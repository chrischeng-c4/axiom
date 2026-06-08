# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "errors"
# case = "verify_named_flags_rejects_unnamed_bit"
# subject = "enum.verify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.verify: @verify(NAMED_FLAGS) on a Flag whose composite member (B=3) covers a bit (0b001) that has no single-bit name raises ValueError"""
import enum


# @verify(NAMED_FLAGS) rejects a Flag whose composite alias covers a bit that
# has no single-bit name: B=3 (0b011) needs a name for bit 0b001, but only
# bit 0b010 and C=4 are named.
_raised = False
try:
    @enum.verify(enum.NAMED_FLAGS)
    class BadFlags(enum.Flag):
        B = 3  # 0b011 — bit 0b001 has no single-bit name
        C = 4
except ValueError:
    _raised = True
assert _raised, "@verify(NAMED_FLAGS) with an unnamed bit must raise ValueError"

print("verify_named_flags_rejects_unnamed_bit OK")
