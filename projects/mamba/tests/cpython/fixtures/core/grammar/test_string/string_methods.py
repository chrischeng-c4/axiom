# RUN: parse
# CPython 3.12 test_string: string methods and operations

# Basic string operations
s = "Hello, World!"
upper = s.upper()
lower = s.lower()
title = s.title()
strip = s.strip()

# Split and join
parts = "a,b,c".split(",")
joined = ",".join(["a", "b", "c"])

# Find and replace
idx = s.find("World")
replaced = s.replace("World", "Python")

# String formatting
name = "world"
formatted = "Hello, %s!" % name
formatted2 = "Hello, {}!".format(name)
formatted3 = f"Hello, {name}!"

# startswith / endswith
starts = s.startswith("Hello")
ends = s.endswith("!")

# String predicates
alpha = "abc".isalpha()
digit = "123".isdigit()
alnum = "abc123".isalnum()
space = "   ".isspace()

# Encode
encoded = s.encode("utf-8")

# Multiline strings
multi = """
line 1
line 2
line 3
"""

raw = r"no \n escape"
