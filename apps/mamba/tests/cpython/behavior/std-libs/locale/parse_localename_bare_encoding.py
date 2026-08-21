# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "locale"
# dimension = "behavior"
# case = "parse_localename_bare_encoding"
# subject = "locale._parse_localename"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_locale.py"
# status = "filled"
# ///
"""locale._parse_localename: _parse_localename of a bare encoding 'UTF-8' yields (None, 'UTF-8')"""
import locale

# _parse_localename splits "<lang>.<encoding>"; a bare encoding yields
# (None, encoding).
assert locale._parse_localename("UTF-8") == (None, "UTF-8"), "_parse_localename UTF-8"

print("parse_localename_bare_encoding OK")
