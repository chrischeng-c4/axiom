# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warningmessage_attribute_types"
# subject = "warnings.WarningMessage"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.WarningMessage: a recorded WarningMessage exposes category (Warning subclass), message (text), lineno (int) and filename (str) with the correct types"""
import warnings

with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    warnings.warn("attr check", RuntimeWarning, stacklevel=1)
    msg = recorded[0]
    assert issubclass(msg.category, RuntimeWarning), f"category = {msg.category!r}"
    assert "attr check" in str(msg.message), f"message = {msg.message!r}"
    assert isinstance(msg.lineno, int), f"lineno type = {type(msg.lineno)!r}"
    assert isinstance(msg.filename, str), f"filename type = {type(msg.filename)!r}"

print("warningmessage_attribute_types OK")
