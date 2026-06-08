# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "enum"
# dimension = "behavior"
# case = "intflag_bitwise_membership"
# subject = "enum.IntFlag"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_enum.py"
# status = "filled"
# ///
"""enum.IntFlag: IntFlag supports |, &, in-membership, and int equality: READ|WRITE|EXEC == 7 and each component is `in` the composite"""
import enum


class Perm(enum.IntFlag):
    READ = 1
    WRITE = 2
    EXEC = 4


rwx = Perm.READ | Perm.WRITE | Perm.EXEC
assert rwx == 7, f"READ|WRITE|EXEC = {rwx!r}"
assert (Perm.READ | Perm.WRITE) & Perm.READ, "& isolates a set bit"
assert Perm.READ in rwx, "READ is in the composite"
assert Perm.WRITE in rwx, "WRITE is in the composite"
assert isinstance(Perm.READ, int), "IntFlag member is also int"

print("intflag_bitwise_membership OK")
