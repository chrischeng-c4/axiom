# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "os"
# dimension = "behavior"
# case = "devnull_discards_writes"
# subject = "os.devnull"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""os.devnull: writing bytes to os.devnull is discarded and reading it back yields empty bytes"""
import os

with open(os.devnull, "wb", 0) as wn:
    wn.write(b"hello")
with open(os.devnull, "rb") as rn:
    assert rn.read() == b"", "reading os.devnull yields empty bytes"
print("devnull_discards_writes OK")
