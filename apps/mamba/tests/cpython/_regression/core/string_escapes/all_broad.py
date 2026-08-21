# string escapes broad

# basic escapes
print("a\nb")
print("a\tb")
print("a\\b")
print("a\"b")
print('a\'b')

# newline in literal
print("line1\nline2\nline3")

# tab
print("col1\tcol2\tcol3")

# hex
print("\x41\x42\x43")
print("\x48\x69")

# unicode (ascii-safe)
print("é")  # é
print("☃")  # snowman

# carriage return
print("abc\rdef")  # actually overwrites

# backslash at end
print("a\\")
print("\\\\")

# null byte (be careful)
s = "a\x00b"
print(len(s))

# raw string
print(r"\n")
print(r"\t")
print(r"\\")
print(r"no interp: \x41")

# triple-quoted
t = """line 1
line 2
line 3"""
print(t)

s2 = '''single'''
print(s2)

# mixed escapes
print("tab:\there\nnewline")

# bytes escapes
print(b"\x48\x65\x6c\x6c\x6f")
print(b"a\nb")
