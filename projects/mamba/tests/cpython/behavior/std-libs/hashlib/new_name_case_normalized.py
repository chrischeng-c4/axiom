# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "new_name_case_normalized"
# subject = "hashlib.new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.new: new() is case-insensitive: new('SHA256').name=='sha256', new('Sha1').name=='sha1', and the uppercase-name digest matches the direct constructor"""
import hashlib

assert hashlib.new("SHA256").name == "sha256", "new('SHA256').name canonicalized"
assert hashlib.new("Sha1").name == "sha1", "new('Sha1').name canonicalized"
assert hashlib.new("SHA256", b"hello").digest() == hashlib.sha256(b"hello").digest(), \
    "uppercase new() == direct"

print("new_name_case_normalized OK")
