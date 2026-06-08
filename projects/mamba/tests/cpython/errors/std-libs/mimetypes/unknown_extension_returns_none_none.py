# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "errors"
# case = "unknown_extension_returns_none_none"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""mimetypes.guess_type: an unregistered extension is not an error: guess_type returns (None, None) rather than raising"""
import mimetypes

_t, _e = mimetypes.guess_type("file.xyz_unknown_ext_123")
assert _t is None, f"unknown type = {_t!r}"
assert _e is None, f"unknown encoding = {_e!r}"
print("unknown_extension_returns_none_none OK")
