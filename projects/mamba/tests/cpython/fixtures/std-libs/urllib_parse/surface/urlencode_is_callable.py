# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "urlencode_is_callable"
# subject = "urllib.parse.urlencode"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.parse.urlencode -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.urlencode: urlencode_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.urlencode)
print("urlencode_is_callable OK")
