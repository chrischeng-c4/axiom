# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# String/repr builtin edge cases
print(repr(42))
print(repr('hello'))
print(repr([1, 2, 3]))
print(repr(None))
print(repr(True))
print(chr(65))
print(chr(8364))
print(ord('A'))
print(ord('\u20ac'))
print(format(3.14159, '.2f'))
print(format(42, '08b'))
