# string escapes and specials broad

# basic escapes
print("line1\nline2")
print("tab\there")
print("backslash: \\")
print("quote: \"hi\"")
print("single: 'hi'")

# unicode escapes
print("snowman: ☃")
print("smile: ☺")

# hex escapes
print("A: \x41")
print("a: \x61")

# raw strings
print(r"raw\nstring")
print(r"C:\Windows\Path")

# multiline strings
s = """first
second
third"""
print(s)

# multiline with escape
s2 = "line1\n" + "line2\n" + "line3"
print(s2)

# string ops
print("hello" + " " + "world")
print("abc" * 3)
print("-" * 10)

# repr escapes special chars
print(repr("with\nnewline"))
print(repr("with\tab"))
print(repr("with 'quote'"))

# len counts chars not bytes
print(len("hello"))
print(len(""))
print(len("\n"))
print(len("\\"))
print(len("a\nb"))

# char access
s = "hello"
print(s[0])
print(s[-1])
print(s[2])

# string iteration
for c in "abc":
    print(c)

# escapes in f-string
x = 5
print(f"line: {x}")

# escape in join
print(",".join(["a", "b", "c"]))
print("\n".join(["a", "b", "c"]))
print("-".join([]))
print("".join(["x", "y", "z"]))

# split on special chars
print("a,b,c".split(","))
print("a\nb\nc".split("\n"))
print("one two three".split(" "))
print("x".split(","))

# contains special
print("\n" in "a\nb")
print("\t" in "no tab here")
print("\\" in "path\\file")
