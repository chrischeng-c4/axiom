# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "new_equals_direct_constructor"
# subject = "hashlib.new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.new: new('sha256', data) produces the same digest as the direct sha256(data) constructor"""
import hashlib

_by_name = hashlib.new("sha256", b"hello")
_by_direct = hashlib.sha256(b"hello")
assert _by_name.digest() == _by_direct.digest(), "new() == direct constructor"

print("new_equals_direct_constructor OK")
