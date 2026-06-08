# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "int_flag_mixin_returns_flag_type"
# subject = "enum.Flag"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.Flag: an (int, Flag) mixin returns instances of the flag type from bitwise ops, even when combined with a bare int, and members are also ints"""
import enum


class Bits(int, enum.Flag):
    ONE = 1
    TWO = 2
    FOUR = 4


assert isinstance(Bits.ONE | Bits.TWO, Bits), "flag | flag returns Bits"
assert isinstance(Bits.ONE | 2, Bits), "flag | bare int returns Bits"
assert isinstance(Bits.ONE, int), "(int, Flag) member is also int"

print("int_flag_mixin_returns_flag_type OK")
