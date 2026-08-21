# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "behavior"
# case = "warn_stacklevel_records_location"
# subject = "warnings.warn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""warnings.warn: warn(msg, UserWarning, stacklevel=2) emitted from a helper still records well-typed lineno (int) and filename (str) location fields"""
import warnings


def emit():
    warnings.warn("from caller", UserWarning, stacklevel=2)


with warnings.catch_warnings(record=True) as recorded:
    warnings.simplefilter("always")
    emit()
    assert isinstance(recorded[0].lineno, int), f"lineno type = {type(recorded[0].lineno)!r}"
    assert isinstance(recorded[0].filename, str), f"filename type = {type(recorded[0].filename)!r}"

print("warn_stacklevel_records_location OK")
