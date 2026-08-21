# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha256_empty_known_vector"
# subject = "hashlib.sha256"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha256: sha256 of the empty input is the fixed constant e3b0c442...b7852b855"""
import hashlib

_empty = hashlib.sha256(b"").hexdigest()
assert _empty == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855", \
    f"sha256('') = {_empty!r}"

print("sha256_empty_known_vector OK")
