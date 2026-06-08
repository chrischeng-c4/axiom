# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "errors"
# case = "rand_bytes_negative_raises"
# subject = "ssl.RAND_bytes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.RAND_bytes: RAND_bytes rejects a negative byte count with ValueError regardless of PRNG seeding state"""
import ssl

try:
    ssl.RAND_bytes(-5)
    raise AssertionError("RAND_bytes(-5) should raise")
except ValueError:
    pass

print("rand_bytes_negative_raises OK")
