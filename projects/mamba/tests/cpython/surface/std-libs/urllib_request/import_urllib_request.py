# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_request"
# dimension = "surface"
# case = "import_urllib_request"
# subject = "urllib.request"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request: import_urllib_request (surface)."""
import urllib.request

assert hasattr(urllib.request, "urlopen")
print("import_urllib_request OK")
