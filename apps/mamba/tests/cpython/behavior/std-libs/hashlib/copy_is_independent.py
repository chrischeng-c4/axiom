# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "copy_is_independent"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: copy() snapshots state: updating the original after copy() leaves the copy's digest equal to the un-updated base"""
import hashlib

_h = hashlib.sha256(b"base")
_c = _h.copy()
_h.update(b"_extra")
_before = _c.hexdigest()
assert _before == hashlib.sha256(b"base").hexdigest(), "copy unaffected by original update"

print("copy_is_independent OK")
