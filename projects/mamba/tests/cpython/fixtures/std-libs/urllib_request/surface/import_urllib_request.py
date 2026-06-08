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
# xfail = "urllib.request unimplemented on mamba: module is a near-empty stub, hasattr(urllib.request, 'urlopen') is False (probed 2026-05-29, mamba 0.3.60; repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request: import_urllib_request (surface)."""
import urllib.request

assert hasattr(urllib.request, "urlopen")
print("import_urllib_request OK")
