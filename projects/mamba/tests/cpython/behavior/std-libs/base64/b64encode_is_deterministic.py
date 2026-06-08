# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "base64"
# dimension = "behavior"
# case = "b64encode_is_deterministic"
# subject = "base64.b64encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""base64.b64encode: b64encode is a pure function: the same input always produces the same output"""
import base64

_d = b"test data 123"
assert base64.b64encode(_d) == base64.b64encode(_d), "deterministic"
print("b64encode_is_deterministic OK")
