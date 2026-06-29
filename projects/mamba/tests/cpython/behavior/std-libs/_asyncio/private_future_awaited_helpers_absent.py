# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_asyncio"
# dimension = "behavior"
# case = "private_future_awaited_helpers_absent"
# subject = "_asyncio private future awaited helpers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/asyncio"
# status = "filled"
# ///
"""CPython 3.12 does not expose private future awaited helper functions from _asyncio."""

import _asyncio


assert not hasattr(_asyncio, "future_add_to_awaited_by")
assert not hasattr(_asyncio, "future_discard_from_awaited_by")
print("private_future_awaited_helpers_absent OK")
