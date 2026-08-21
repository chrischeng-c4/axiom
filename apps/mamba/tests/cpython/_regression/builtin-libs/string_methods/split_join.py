# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
print("a,b,c".split(","))
print("a,b,c".split(",", 1))
print(",".join(["a", "b", "c"]))
print("hello world".split())
print("  hello  world  ".split())
print("a::b::c".split("::"))
print(" ".join(["hello", "world"]))
try:
    result = "abc".split("")
    print(result)
except ValueError as e:
    print("caught ValueError")
