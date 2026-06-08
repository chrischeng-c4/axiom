# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "behavior"
# case = "rand_add_accepts_entropy"
# subject = "ssl.RAND_add"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_ssl.py"
# status = "filled"
# ///
"""ssl.RAND_add: RAND_add accepts str and bytes-like entropy (str, bytes, bytearray) without raising"""
import ssl

ssl.RAND_add("this is a random string", 75.0)
ssl.RAND_add(b"this is a random bytes object", 75.0)
ssl.RAND_add(bytearray(b"this is a random bytearray object"), 75.0)

print("rand_add_accepts_entropy OK")
