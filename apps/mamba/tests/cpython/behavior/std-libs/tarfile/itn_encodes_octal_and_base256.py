# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "itn_encodes_octal_and_base256"
# subject = "tarfile.itn"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.itn: itn writes the octal form by default (1, 2**21-1) and the GNU base-256 form for values that overflow octal or are negative (2**21, -1, -100)"""
import tarfile

# Default form is octal.
assert tarfile.itn(1) == b"0000001\x00", "itn 1"
assert tarfile.itn(2097151) == b"7777777\x00", "itn octal max"

# Values that overflow octal need GNU base-256 (returns a bytearray, which
# compares equal to bytes).
assert tarfile.itn(2097152, format=tarfile.GNU_FORMAT) == b"\x80\x00\x00\x00\x00 \x00\x00", "itn gnu 2**21"
assert tarfile.itn(-1, format=tarfile.GNU_FORMAT) == b"\xff\xff\xff\xff\xff\xff\xff\xff", "itn gnu -1"
assert tarfile.itn(-100, format=tarfile.GNU_FORMAT) == b"\xff\xff\xff\xff\xff\xff\xff\x9c", "itn gnu -100"

print("itn_encodes_octal_and_base256 OK")
