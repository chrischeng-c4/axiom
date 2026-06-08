# RUN: parse
# CPython 3.12 test_string: raw strings and byte strings

# Raw strings
r1 = r"no \n escape"
r2 = r'single \t quote'
r3 = r"""triple \r\n raw"""

# Byte strings
b1 = b"hello"
b2 = b'world'
b3 = b"""multi
line
bytes"""

# Raw byte strings
rb1 = rb"raw \n bytes"
rb2 = br"also raw \t bytes"

# String prefixes
u1 = u"unicode"

# Byte operations
concat = b"hello" + b" " + b"world"
repeat = b"ab" * 3
length = len(b"hello")
