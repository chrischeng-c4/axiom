# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
print("hello".startswith("hel"))
print("hello".startswith("world"))
print("hello".endswith("llo"))
print("hello".endswith("hel"))
print("hello".startswith(("hel", "wor")))
print("hello".startswith(("xyz", "wor")))
print("hello".endswith(("llo", "xyz")))
print("hello".startswith("", 0))
print("hello".startswith("ell", 1))
