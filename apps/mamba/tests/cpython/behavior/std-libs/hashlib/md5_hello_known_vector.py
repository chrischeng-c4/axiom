# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "md5_hello_known_vector"
# subject = "hashlib.md5"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.md5: md5(b'hello') hexdigest is the reference 5d41402abc4b2a76b9719d911017c592"""
import hashlib

_md5_hello = hashlib.md5(b"hello").hexdigest()
assert _md5_hello == "5d41402abc4b2a76b9719d911017c592", f"md5('hello') = {_md5_hello!r}"

print("md5_hello_known_vector OK")
