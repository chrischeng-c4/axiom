# RUN: parse
# CPython-derived: complex string literal patterns (#561)

# --- raw strings ---
s = r"no escape \n here"
s = r'single raw \t string'
s = r"backslash \\ stays"
s = r"regex pattern: \d+\.\d+"

# --- raw triple-quoted ---
s = r"""raw triple
with \n no escapes"""
s = r'''raw triple single
with \t literal tabs'''

# --- byte strings ---
s = b"byte string"
s = b'single byte'
s = b"with escapes \n\t"
s = b"\x00\x01\x02\xff"

# --- byte triple-quoted ---
s = b"""triple
byte string"""
s = b'''triple single
byte string'''

# --- raw byte strings (both prefix orders) ---
s = rb"raw bytes \n"
s = br"raw bytes \t"
s = rb'single raw bytes \n'
s = br'single raw bytes \t'

# --- raw byte triple-quoted ---
s = rb"""raw triple bytes
\n literal"""
s = br'''raw triple bytes
\t literal'''

# --- triple-quoted strings ---
s = """multi
line
string"""

s = '''single
triple
quoted'''

# --- triple-quoted with quotes inside ---
s = """contains "double" quotes"""
s = '''contains 'single' quotes'''
s = """contains both "double" and 'single' quotes"""

# --- string concatenation (implicit) ---
s = "hello " "world"
s = "one" "two" "three"
# NOTE: implicit multi-line string concat in parens not supported
s = "part1" + "part2" + "part3"


# --- byte string concatenation ---
s = b"hello " b"world"
s = b"one" b"two" b"three"

# --- unicode escapes ---
s = "\u0041"
s = "\U00000041"
s = "\N{LATIN SMALL LETTER A}"
s = "\N{GREEK SMALL LETTER ALPHA}"
s = "\u00e9\u00e8\u00ea"

# --- hex and octal escapes ---
s = "\x41\x42\x43"
s = "\101\102\103"
s = b"\x00\x0a\x0d\xff"

# --- common escape sequences ---
s = "newline\n"
s = "tab\t"
s = "backslash\\"
s = "single\'"
s = "double\""
s = "carriage\r"
s = "formfeed\f"
s = "backspace\b"
s = "bell\a"
s = "vertical\v"
s = "null\0"

# --- null bytes in bytes ---
s = b"\x00"
s = b"\x00\x00\x00"
s = b"before\x00after"

# --- multi-line strings with continuation ---
# NOTE: backslash continuation in string literal not supported
# s = "first line \n# second line"

# s = 'first line \n# second line'
# --- very long string (multi-line via triple quote) ---
s = """
This is a long string that spans
multiple lines for testing the parser
ability to handle extended string literals
without any issues at all.
Line five of the string.
Line six of the string.
Line seven of the string.
"""

# --- f-string with raw ---
x = 42
s = rf"raw f-string {x} with \n literal"
s = rf'\t {x} \n'
s = rf"""multi-line raw f
{x} with \n"""

# --- fr prefix order ---
s = fr"also valid {x}"

# --- empty strings of all types ---
s = ""
s = ''
s = b""
s = b''
s = r""
s = r''
s = rb""
s = rb''
s = br""
s = br''
s = f""
s = f''
s = rf""
s = rf''

# --- empty triple-quoted ---
s = """"""
s = ''''''
s = b""""""
s = b''''''

# --- string with only whitespace ---
s = "   "
s = "\t\n\r"
s = "  \t  \n  "

# --- strings with various quote nesting ---
s = "it's a string"
s = 'it "is" a string'
s = """it's "both" types"""
s = '''it's "both" types too'''

# --- bytes with all byte values ---
s = b"\x00\x7f\x80\xff"
s = b"\t\n\r\x1b"

# --- string repetition in literal context ---
s = "ab" * 3
s = b"cd" * 2

# --- string in collection context ---
lst = ["a", "b", "c"]
tup = ("x", "y", "z")
st = {"p", "q", "r"}
dct = {"key": "value", "k2": "v2"}

# --- multi-line string concatenation ---
# NOTE: implicit multi-line string concat in parens not supported
# s = (
#     "part one "
#     "part two "
#     "part three"
# )
s = "part one " + "part two " + "part three"

# --- byte string with escape sequences ---
s = b"line1\nline2\ttab"
s = b"quote\"inside"
s = b"null\x00byte"

# --- string with backslash at various positions ---
s = "\\start"
s = "mid\\dle"
s = "end\\"

# --- raw string edge cases ---
s = r"ends with space "
s = r""
s = r"\\"

# --- mixed raw and regular ---
s = r"\n" + "\n"
s = r"\t" + "\t"
