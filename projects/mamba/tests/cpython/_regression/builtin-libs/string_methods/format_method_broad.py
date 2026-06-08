# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string format method broad (.format)

# positional
print("{} {}".format("hello", "world"))
print("{}-{}".format(1, 2))
print("{}+{}={}".format(1, 2, 3))

# positional indexed
print("{0}/{1}".format("a", "b"))
print("{1}/{0}".format("a", "b"))
print("{0}-{0}-{1}".format("x", "y"))

# keyword
print("{name}".format(name="alice"))
print("{a}+{b}={c}".format(a=1, b=2, c=3))
print("{greeting}, {name}".format(greeting="hi", name="bob"))

# mix keyword and positional
print("{} + {b}".format(1, b=2))
print("{0} / {name}".format("top", name="slash"))

# format int with width
print("{:5d}".format(42))
print("{:05d}".format(42))
print("{:<5d}".format(42))
print("{:>5d}".format(42))

# format float precision
print("{:.2f}".format(3.14159))
print("{:.0f}".format(3.7))

# format string width
print("{:>10}".format("hi"))
print("{:<10}".format("hi"))

# fill char
print("{:*>10}".format("hi"))
print("{:*<10}".format("hi"))

# nested braces (literal)
print("{{literal}}")
print("{{{}}}".format("x"))

# multiple format in one
print("[{}, {}, {}, {}]".format(1, 2, 3, 4))
print("{}:{}:{}".format("h", "m", "s"))
