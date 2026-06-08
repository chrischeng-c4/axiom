# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "tempfile"
# dimension = "behavior"
# case = "named_file_iterates_lines"
# subject = "tempfile.NamedTemporaryFile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_tempfile.py"
# status = "filled"
# ///
"""tempfile.NamedTemporaryFile: iterating a NamedTemporaryFile(mode='w+b') after seek(0) yields its lines in order"""
import tempfile

lines = [b"spam\n", b"eggs\n", b"beans\n"]
f = tempfile.NamedTemporaryFile(mode="w+b")
f.write(b"".join(lines))
f.seek(0)
seen = []
for line in f:
    seen.append(line)
assert seen == lines, f"iterated lines = {seen!r}"
f.close()
print("named_file_iterates_lines OK")
