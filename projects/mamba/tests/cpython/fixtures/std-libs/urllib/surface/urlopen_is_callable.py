# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "urlopen_is_callable"
# subject = "urllib.request.urlopen"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.request.urlopen -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.request.urlopen: urlopen_is_callable (surface)."""
import urllib.request

assert callable(urllib.request.urlopen)
print("urlopen_is_callable OK")
