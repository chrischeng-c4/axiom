# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "spooled_generic_alias_subscription"
# subject = "tempfile.SpooledTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.SpooledTemporaryFile: SpooledTemporaryFile[bytes] produces a types.GenericAlias"""
import types
import tempfile

_alias = tempfile.SpooledTemporaryFile[bytes]
assert isinstance(_alias, types.GenericAlias), f"alias = {type(_alias)!r}"
print("spooled_generic_alias_subscription OK")
