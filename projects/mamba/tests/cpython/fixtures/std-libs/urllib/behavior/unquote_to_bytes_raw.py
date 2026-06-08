# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib"
# dimension = "behavior"
# case = "unquote_to_bytes_raw"
# subject = "urllib.parse.unquote_to_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_urllib.py"
# status = "filled"
# ///
"""urllib.parse.unquote_to_bytes: unquote_to_bytes returns raw bytes regardless of encodability, accepting both str and bytes input"""
from urllib.parse import unquote_to_bytes

assert unquote_to_bytes("br%C3%BCckner") == b"br\xc3\xbcckner", "unquote_to_bytes"
assert unquote_to_bytes(b"%A2%D8ab%FF") == b"\xa2\xd8ab\xff", "unquote_to_bytes(bytes)"

print("unquote_to_bytes_raw OK")
