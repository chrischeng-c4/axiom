# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "algorithms_guaranteed_set_contents"
# subject = "hashlib.algorithms_guaranteed"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.algorithms_guaranteed: algorithms_guaranteed is a set/frozenset of names containing at least 'sha256' and 'md5'"""
import hashlib

_alg = hashlib.algorithms_guaranteed
assert isinstance(_alg, (set, frozenset)), f"algorithms_guaranteed type = {type(_alg)!r}"
assert "sha256" in _alg, "sha256 in guaranteed"
assert "md5" in _alg, "md5 in guaranteed"

print("algorithms_guaranteed_set_contents OK")
