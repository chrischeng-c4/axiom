# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "install_opener_is_callable"
# subject = "urllib.request.install_opener"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.install_opener: install_opener_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.install_opener)
print("install_opener_is_callable OK")
