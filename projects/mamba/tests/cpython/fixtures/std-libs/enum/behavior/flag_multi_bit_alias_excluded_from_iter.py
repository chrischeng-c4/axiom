# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "flag_multi_bit_alias_excluded_from_iter"
# subject = "enum.Flag"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Flag: a multi-bit Flag value (B=3, D=6) is an alias of its single-bit members, retains its .value, and is excluded from canonical iteration (only C=4 is canonical)"""
import enum


class Bizarre(enum.Flag):
    B = 3
    C = 4
    D = 6


# Only the single-bit member C is canonical; B and D are multi-bit aliases.
assert list(Bizarre) == [Bizarre.C], f"canonical = {list(Bizarre)!r}"
assert Bizarre.B.value == 3, "aliased value retained"
assert Bizarre.D.value == 6, "aliased value retained"

print("flag_multi_bit_alias_excluded_from_iter OK")
