# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binascii"
# dimension = "behavior"
# case = "hexlify_separator_and_group"
# subject = "binascii.hexlify"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_binascii.py"
# status = "filled"
# ///
"""binascii.hexlify: hexlify(sep) and hexlify(sep, bytes_per_group) match bytes.hex"""
import binascii

# A bare separator inserts one between every byte.
_hex_sep = binascii.hexlify(b"\x01\x02\x03", ":")
assert isinstance(_hex_sep, bytes), f"hexlify sep type = {type(_hex_sep)!r}"
assert _hex_sep == b"01:02:03", f"hexlify with sep = {_hex_sep!r}"

# A separator plus a bytes-per-group count groups from the right.
_payload = bytes(range(1, 11))
_grouped = binascii.hexlify(_payload, ".", 4)
assert _grouped == b"0102.03040506.0708090a", f"grouped hex = {_grouped!r}"
assert _grouped == _payload.hex(".", 4).encode("ascii"), "matches bytes.hex"

print("hexlify_separator_and_group OK")
