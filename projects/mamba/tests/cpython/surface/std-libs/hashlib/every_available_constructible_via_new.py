# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "surface"
# case = "every_available_constructible_via_new"
# subject = "hashlib.new"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""hashlib.new: every advertised algorithm in algorithms_available is constructible via new(name, usedforsecurity=False) without raising"""
import hashlib

for _name in hashlib.algorithms_available:
    _probe = hashlib.new(_name, usedforsecurity=False)
    assert _probe is not None, f"new({_name!r}, usedforsecurity=False)"

print("every_available_constructible_via_new OK")
