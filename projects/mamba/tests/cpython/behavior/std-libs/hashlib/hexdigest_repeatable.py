# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "hexdigest_repeatable"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: hexdigest() does not consume state (repeatable) and digest().hex() equals hexdigest()"""
import hashlib

_rep = hashlib.sha256(b"abc")
assert _rep.hexdigest() == _rep.hexdigest(), "hexdigest repeatable"
assert _rep.digest().hex() == _rep.hexdigest(), "digest().hex() == hexdigest()"

print("hexdigest_repeatable OK")
