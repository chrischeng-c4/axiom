# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# String predicate methods that Mamba was missing: isascii, isidentifier,
# isnumeric, isdecimal, isprintable. CPython exposes them all as `str` methods
# returning bool. Mamba previously raised AttributeError for the first three;
# the fix wires `mb_str_is{ascii,identifier,numeric,decimal,printable}` into
# the str-method dispatch in `runtime/string_ops.rs::mb_str_call_method`.

# isascii — True for ASCII-only strings (incl. empty), False otherwise.
print("abc".isascii())                 # True
print("".isascii())                    # True
print("héllo".isascii())               # False
print("hello\nworld".isascii())        # True (newline is ASCII)

# isidentifier — first char letter/underscore, rest alnum/underscore.
print("abc".isidentifier())            # True
print("_x1".isidentifier())            # True
print("3abc".isidentifier())           # False (leading digit)
print("abc def".isidentifier())        # False (space)
print("".isidentifier())               # False
print("foo_bar".isidentifier())        # True

# isnumeric / isdecimal — both stricter than isdigit; we use Unicode rules.
print("123".isnumeric())               # True
print("123".isdecimal())               # True
print("12.3".isnumeric())              # False (dot)
print("".isnumeric())                  # False

# isprintable — empty is True; whitespace is False except space.
print("hello".isprintable())           # True
print("hello world".isprintable())     # True (space is printable)
print("hello\tworld".isprintable())    # False (tab)
print("".isprintable())                # True
