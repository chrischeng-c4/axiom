# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "update_equivalent_to_concatenation"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: incremental update() chained over 'hello'+'world' equals the one-shot digest of 'helloworld'"""
import hashlib

_h_cat = hashlib.sha256(b"helloworld")
_h_inc = hashlib.sha256()
_h_inc.update(b"hello")
_h_inc.update(b"world")
assert _h_cat.digest() == _h_inc.digest(), "update equivalent to cat"

print("update_equivalent_to_concatenation OK")
