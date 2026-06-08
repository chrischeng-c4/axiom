# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "switchinterval_roundtrip"
# subject = "sys.setswitchinterval"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sys.setswitchinterval: the switch interval is a small positive float (< 0.5) that survives a setswitchinterval(0.05)/getswitchinterval round trip, then restores"""
import sys

orig = sys.getswitchinterval()
assert orig < 0.5, f"default switchinterval = {orig!r}"
try:
    sys.setswitchinterval(0.05)
    assert abs(sys.getswitchinterval() - 0.05) < 1e-7, "switchinterval round trip"
finally:
    sys.setswitchinterval(orig)
assert abs(sys.getswitchinterval() - orig) < 1e-7, "switchinterval restored"
print("switchinterval_roundtrip OK")
