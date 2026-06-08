# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# String/repr builtins: chr, ord, repr of special characters
# String/repr builtins conformance (S6-S7)
# repr, chr, ord

# S6: repr for all types
print(repr(42))
print(repr('hello'))
print(repr([1, 2]))
print(repr(None))
print(repr(True))
print(repr(False))
print(repr(3.14))
print(repr((1, 2, 3)))
print(repr({'a': 1}))
print(repr(set()))

# repr of special values
print(repr(''))
print(repr([]))
print(repr({}))
print(repr(()))
print(repr(0))
print(repr(-1))
print(repr(0.0))

# repr of strings with special characters
print(repr("it's"))
print(repr('he said "hi"'))
print(repr('line1\nline2'))
print(repr('tab\there'))

# S7: chr/ord round-trip
print(chr(65))
print(chr(8364))
print(ord('A'))
print(ord('\u20ac'))

# chr/ord additional
print(chr(48))
print(chr(122))
print(chr(32))
print(ord('0'))
print(ord('z'))
print(ord('\n'))

# chr/ord round-trip verification
print(chr(ord('A')) == 'A')
print(ord(chr(65)) == 65)
print(chr(ord('\u20ac')) == '\u20ac')
