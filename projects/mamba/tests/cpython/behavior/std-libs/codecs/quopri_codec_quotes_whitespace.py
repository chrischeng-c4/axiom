# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "codecs"
# dimension = "behavior"
# case = "quopri_codec_quotes_whitespace"
# subject = "codecs.encode"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_codecs.py"
# status = "filled"
# ///
"""codecs.encode: quopri-codec quotes whitespace: codecs.encode(b'space tab\\teol \\n','quopri-codec') is b'space=20tab=09eol=20\\n', and a plain line decodes unchanged"""
import codecs

_q = codecs.encode(b"space tab\teol \n", "quopri-codec")
assert _q == b"space=20tab=09eol=20\n", f"quopri encode = {_q!r}"
_plain = b"space tab eol\n"
assert codecs.decode(_plain, "quopri-codec") == _plain, "quopri decode passthrough"

print("quopri_codec_quotes_whitespace OK")
