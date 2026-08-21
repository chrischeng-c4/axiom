# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mimetypes"
# dimension = "behavior"
# case = "guess_type_encoding_layer"
# subject = "mimetypes.guess_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_mimetypes.py"
# status = "filled"
# ///
"""mimetypes.guess_type: a compression suffix yields the encoding: file.gz -> (None, 'gzip') and archive.tar.gz -> ('application/x-tar', 'gzip')"""
import mimetypes

# A bare .gz carries the encoding but no underlying type.
t1, e1 = mimetypes.guess_type("file.gz")
assert t1 is None, f"gz type = {t1!r}"
assert e1 == "gzip", f"gz encoding = {e1!r}"

# A compound .tar.gz resolves to the tar type plus the gzip encoding.
t2, e2 = mimetypes.guess_type("archive.tar.gz")
assert t2 == "application/x-tar", f"tar.gz type = {t2!r}"
assert e2 == "gzip", f"tar.gz encoding = {e2!r}"
print("guess_type_encoding_layer OK")
