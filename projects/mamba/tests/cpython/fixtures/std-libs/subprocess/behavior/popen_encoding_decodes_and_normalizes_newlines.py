# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "popen_encoding_decodes_and_normalizes_newlines"
# subject = "subprocess.Popen"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
"""subprocess.Popen: Popen(encoding=...) decodes child output in the given codec and normalizes CR/CRLF newlines to '\\n' (utf-16 and utf-32-be)"""
import subprocess
import sys

for encoding in ("utf-16", "utf-32-be"):
    code = "import sys; sys.stdout.buffer.write('1\\r\\n2\\r3\\n4'.encode('%s'))" % encoding
    p = subprocess.Popen(
        [sys.executable, "-c", code],
        stdin=subprocess.PIPE, stdout=subprocess.PIPE, encoding=encoding,
    )
    out, _ = p.communicate(input="")
    assert out == "1\n2\n3\n4", f"{encoding} decoded = {out!r}"
print("popen_encoding_decodes_and_normalizes_newlines OK")
