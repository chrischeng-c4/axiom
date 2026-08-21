# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "utf8_sig_strips_bom"
# subject = "codecs.getincrementaldecoder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getincrementaldecoder: the utf-8-sig incremental decoder transparently strips a leading BOM: decoding 'spam'.encode('utf-8-sig') yields 'spam'"""
import codecs

_sig = codecs.getincrementaldecoder("utf-8-sig")()
assert _sig.decode("spam".encode("utf-8-sig")) == "spam"

print("utf8_sig_strips_bom OK")
