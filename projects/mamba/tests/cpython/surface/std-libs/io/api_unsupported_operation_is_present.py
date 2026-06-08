# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "io"
# dimension = "surface"
# case = "api_unsupported_operation_is_present"
# subject = "io.UnsupportedOperation"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "projects/mamba/data/cpython312_surface.json"
# status = "filled"
# ///
"""io.UnsupportedOperation: api_unsupported_operation_is_present (surface)."""
import io

assert hasattr(io, "UnsupportedOperation")
print("api_unsupported_operation_is_present OK")
