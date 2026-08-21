# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "errorcode_values_are_attrs"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
"""errno.errorcode: every name in errorcode.values() is a real module attribute whose int round-trips back to the same errorcode key"""
import errno

for code, name in errno.errorcode.items():
    assert hasattr(errno, name), f"errorcode value {name!r} missing as attr"
    assert getattr(errno, name) == code, f"attr {name} != errorcode key {code}"
print("errorcode_values_are_attrs OK")
