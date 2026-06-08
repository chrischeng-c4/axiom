# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""bytes/bytearray %-formatting: %b, %s, %d, %a, %% and tuple args."""

# %b inserts a bytes-like value verbatim.
assert b"hello, %b!" % b"world" == b"hello, world!"
# %s behaves like %b for bytes payloads.
assert b"%s / 100 = %d%%" % (b"seventy-nine", 79) == b"seventy-nine / 100 = 79%"
# A literal %% becomes a single percent.
assert b"100%%" % () == b"100%"
# %d / %x / %o format integers.
assert b"%d" % 42 == b"42"
assert b"%x" % 255 == b"ff"
assert b"%o" % 8 == b"10"
# Width / zero-pad flags work on integers.
assert b"%05d" % 42 == b"00042"
assert b"%-5d|" % 42 == b"42   |"
# %c accepts an int code point or a length-1 bytes value.
assert b"%c%c" % (65, b"B") == b"AB"
# NUL bytes survive formatting.
assert b"x\x00%b" % b"y" == b"x\x00y"
# %a gives the ascii-repr of the argument as bytes.
assert b"%a" % b"hi" == b"b'hi'"

# %= mutates the binding to a fresh object (bytes are immutable).
acc = b"hello, %b!"
orig = acc
acc %= b"world"
assert acc == b"hello, world!"
assert orig == b"hello, %b!"   # original untouched
assert acc is not orig
assert type(acc) is bytes

# bytearray supports the same operator and stays a bytearray.
ba = bytearray(b"n=%d")
ba %= 7
assert ba == b"n=7"
assert type(ba) is bytearray

print("percent_format OK")
