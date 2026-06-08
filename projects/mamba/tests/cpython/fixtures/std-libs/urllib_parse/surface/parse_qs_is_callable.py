# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_parse"
# dimension = "surface"
# case = "parse_qs_is_callable"
# subject = "urllib.parse.parse_qs"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.parse.parse_qs -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.parse_qs: parse_qs_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.parse_qs)
print("parse_qs_is_callable OK")
