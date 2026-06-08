# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "rand_status_and_bytes"
# subject = "ssl.RAND_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.RAND_bytes: RAND_status() returns an int; when seeded RAND_bytes(16) returns exactly 16 distinct bytes across draws, and when unseeded it raises SSLError"""
import ssl

_status = ssl.RAND_status()
assert isinstance(_status, int), f"RAND_status type = {type(_status)!r}"
if _status:
    _data = ssl.RAND_bytes(16)
    assert isinstance(_data, bytes), "RAND_bytes returns bytes"
    assert len(_data) == 16, f"RAND_bytes(16) length = {len(_data)}"
    assert ssl.RAND_bytes(16) != ssl.RAND_bytes(16), "draws differ"
else:
    try:
        ssl.RAND_bytes(16)
        raise AssertionError("RAND_bytes on unseeded PRNG should raise")
    except ssl.SSLError:
        pass

print("rand_status_and_bytes OK")
