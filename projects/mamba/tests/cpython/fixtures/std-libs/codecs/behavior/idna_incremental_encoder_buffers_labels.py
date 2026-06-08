# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "idna_incremental_encoder_buffers_labels"
# subject = "codecs.getincrementalencoder"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.getincrementalencoder: the incremental IDNA encoder buffers per-label and flushes on a dot or final=True: encode('äx') buffers, encode('ample.org') flushes the first label, encode('',True) flushes the last"""
import codecs

_enc = codecs.getincrementalencoder("idna")()
assert _enc.encode("äx") == b"", "no dot yet -> buffer"
assert _enc.encode("ample.org") == b"xn--xample-9ta.", "first label flushed at dot"
assert _enc.encode("", True) == b"org", "final flush of the last label"
# iterencode over a whole dotted name yields the same bytes.
assert b"".join(codecs.iterencode("pythön.org.", "idna")) == b"xn--pythn-mua.org."

print("idna_incremental_encoder_buffers_labels OK")
