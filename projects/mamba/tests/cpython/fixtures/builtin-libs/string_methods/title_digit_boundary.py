# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `str.title()` should upper-case every cased character that follows a
# non-cased character. CPython treats digits, punctuation, whitespace and
# apostrophes as non-cased, so the letter immediately after each of them
# starts a new "title word":
#   "123abc"   -> "123Abc"
#   "don't"    -> "Don'T"
#   "1a2b3c"   -> "1A2B3C"
#
# Mamba's old implementation tracked a `capitalize_next` flag and triggered
# it on whitespace + non-alphanumeric only. Digits weren't classified as
# word boundaries, so the trailing letters in "123abc" were lower-cased
# instead of becoming "Abc".
#
# Fix in `runtime/string_ops.rs::mb_str_title`: rewrite the state to
# `prev_cased` (alphabetic), upper-case the current char iff cased AND
# !prev_cased, lower-case it iff cased AND prev_cased, otherwise pass it
# through verbatim.

# Headline cases.
print("hello world".title())                    # 'Hello World'
print("HELLO WORLD".title())                    # 'Hello World'
print("hello, world!".title())                  # 'Hello, World!'

# Apostrophes are non-cased — letter after `'` starts a new word.
print("don't".title())                           # "Don'T"
print("can't stop won't stop".title())           # "Can'T Stop Won'T Stop"

# Digits are non-cased — letter after a digit starts a new word.
print("123abc".title())                          # '123Abc'
print("1a2b3c".title())                          # '1A2B3C'
print("python3".title())                         # 'Python3'
print("py3.12".title())                          # 'Py3.12'

# Empty + already-cased correctly.
print("".title())                                # ''
print("hELLO wORLD".title())                     # 'Hello World'

# Non-ASCII still works.
print("ñoño".title())                            # 'Ñoño'
print("café société".title())                    # 'Café Société'
