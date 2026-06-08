# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tarfile"
# dimension = "behavior"
# case = "nti_decodes_octal_and_base256"
# subject = "tarfile.nti"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tarfile.py"
# status = "filled"
# ///
"""tarfile.nti: nti reads octal-ASCII number fields (1, 2**21-1, 0 for nul/spaces) and GNU base-256 fields with the high bit set (2**21, -1, -100)"""
import tarfile

# Octal-ASCII form.
assert tarfile.nti(b"0000001\x00") == 1, "octal 1"
assert tarfile.nti(b"7777777\x00") == 2097151, "octal max (2**21-1)"
assert tarfile.nti(b"\x00") == 0, "single nul -> 0"
assert tarfile.nti(b"       \x00") == 0, "all spaces -> 0"

# GNU base-256 form (high bit of byte 0 set) reaches values octal cannot,
# including negatives.
assert tarfile.nti(b"\x80\x00\x00\x00\x00 \x00\x00") == 2097152, "base-256 2**21"
assert tarfile.nti(b"\xff\xff\xff\xff\xff\xff\xff\xff") == -1, "base-256 -1"
assert tarfile.nti(b"\xff\xff\xff\xff\xff\xff\xff\x9c") == -100, "base-256 -100"

print("nti_decodes_octal_and_base256 OK")
