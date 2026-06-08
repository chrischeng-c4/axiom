# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "surface"
# case = "parse_qsl_is_callable"
# subject = "urllib.parse.parse_qsl"
# kind = "mechanical"
# xfail = "mamba dotted-import quirk: urllib.parse.parse_qsl -> None (repo-memory project_mamba_dotted_import_quirk)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""urllib.parse.parse_qsl: parse_qsl_is_callable (surface)."""
import urllib.parse

assert callable(urllib.parse.parse_qsl)
print("parse_qsl_is_callable OK")
