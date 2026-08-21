# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "sha1_hello_known_vector"
# subject = "hashlib.sha1"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.sha1: sha1(b'hello') hexdigest is the reference aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d"""
import hashlib

_sha1_hello = hashlib.sha1(b"hello").hexdigest()
assert _sha1_hello == "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d", f"sha1('hello') = {_sha1_hello!r}"

print("sha1_hello_known_vector OK")
