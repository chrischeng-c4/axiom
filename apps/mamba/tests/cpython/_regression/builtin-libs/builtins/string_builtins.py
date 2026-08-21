# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Builtins conformance: string-related builtins (R5).
# str, repr, format, chr, ord, ascii

# str
print(str(42))
print(str(3.14))
print(str(True))
print(str(False))
print(str(None))
print(str([1, 2, 3]))
print(str((1, 2)))

# repr
print(repr("hello"))
print(repr("line\nnew"))
print(repr(42))
print(repr(3.14))
print(repr(True))
print(repr(None))
print(repr([1, 2, 3]))

# format
print(format(42, "d"))
print(format(42, "08b"))
print(format(3.14159, ".2f"))
print(format(1000000, ","))
print(format("hello", ">10"))
print(format("hello", "<10"))
print(format("hello", "^10"))

# chr / ord
print(chr(65))
print(chr(97))
print(chr(48))
print(chr(8364))
print(ord("A"))
print(ord("a"))
print(ord("0"))

# ascii
print(ascii("hello"))
print(ascii("caf\u00e9"))
print(ascii("\u0000"))
print(ascii([1, "hello", None]))
