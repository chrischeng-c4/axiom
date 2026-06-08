# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "errno"
# dimension = "behavior"
# case = "uppercase_attrs_are_errorcode_keys"
# subject = "errno.errorcode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_errno.py"
# status = "filled"
# ///
"""errno.errorcode: every uppercase module attribute is an int whose value appears as a key in errorcode"""
import errno

upper_attrs = [a for a in errno.__dict__ if a.isupper()]
assert upper_attrs, "expected some uppercase errno constants"
for attr in upper_attrs:
    value = getattr(errno, attr)
    assert isinstance(value, int), f"{attr} should be int"
    assert value in errno.errorcode, f"{attr}={value} absent from errorcode"
print("uppercase_attrs_are_errorcode_keys OK")
