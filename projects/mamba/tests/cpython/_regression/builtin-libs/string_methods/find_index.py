# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
s = "hello world hello"
print(s.find("hello"))
print(s.rfind("hello"))
print(s.find("xyz"))
print(s.count("hello"))
print(s.count("l"))
print(s.find("hello", 1))
print(s.find("world"))
print(s.index("world"))
try:
    s.index("xyz")
except ValueError:
    print("caught ValueError")
