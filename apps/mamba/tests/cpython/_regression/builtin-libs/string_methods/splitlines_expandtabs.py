# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# str.splitlines, str.partition/rpartition

# splitlines — basic \n splitting works
s = "line1\nline2\nline3"
print(s.splitlines())

# Empty string
print("".splitlines())

# Single line without newline
print("hello".splitlines())

# Trailing newline doesn't add empty
print("hello\n".splitlines())

# partition — returns (before, sep, after)
print("hello.world".partition("."))
print("no-sep-here".partition(":"))
print("a:b:c".partition(":"))

# rpartition — right-to-left
print("a:b:c".rpartition(":"))
print("no-sep".rpartition(":"))

# splitlines — \r alone and \r\n
print("a\rb\rc".splitlines())
print("a\r\nb\r\nc".splitlines())
print("mixed\nand\rand\r\nend".splitlines())

# splitlines(keepends=True)
print("one\ntwo\n".splitlines(True))
print("a\r\nb\r\n".splitlines(True))

# expandtabs — default tabsize=8
print("a\tb".expandtabs())
print("ab\tcd".expandtabs())
print("12345678\tX".expandtabs())

# expandtabs with custom tabsize
print("a\tb".expandtabs(4))
print("\tX".expandtabs(2))
print("a\tb\tc".expandtabs(4))
