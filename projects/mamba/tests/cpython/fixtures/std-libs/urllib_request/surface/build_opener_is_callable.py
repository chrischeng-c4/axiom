# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "build_opener_is_callable"
# subject = "urllib.request.build_opener"
# kind = "mechanical"
# xfail = "urllib.request unimplemented on mamba: urllib.request.build_opener resolves to None/stub (probed 2026-05-29, mamba 0.3.60)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.build_opener: build_opener_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.build_opener)
print("build_opener_is_callable OK")
