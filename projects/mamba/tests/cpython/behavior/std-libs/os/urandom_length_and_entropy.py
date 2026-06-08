# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "urandom_length_and_entropy"
# subject = "os.urandom"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.urandom: os.urandom(n) returns exactly n bytes for several n, and two independent 16-byte draws differ"""
import os

# urandom(n) returns exactly n bytes.
for n in (0, 1, 10, 100, 1000):
    data = os.urandom(n)
    assert isinstance(data, bytes), f"urandom({n}) type = {type(data)!r}"
    assert len(data) == n, f"urandom({n}) len = {len(data)}"

# Two independent draws of meaningful length should differ.
a = os.urandom(16)
b = os.urandom(16)
assert isinstance(a, bytes) and isinstance(b, bytes), "draws are bytes"
assert a != b, "two 16-byte draws collided (astronomically unlikely)"
print("urandom_length_and_entropy OK")
