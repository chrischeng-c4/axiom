# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string startswith/endswith broad

# startswith
print("hello world".startswith("hello"))
print("hello world".startswith("world"))
print("hello world".startswith(""))
print("hello world".startswith("h"))
print("".startswith(""))
print("".startswith("x"))

# endswith
print("hello world".endswith("world"))
print("hello world".endswith("hello"))
print("hello world".endswith(""))
print("hello world".endswith("d"))
print("".endswith(""))

# case sensitive
print("Hello".startswith("hello"))
print("Hello".startswith("Hello"))
print("WORLD".endswith("world"))
print("WORLD".endswith("WORLD"))

# with start index
print("hello world".startswith("world", 6))
print("hello world".startswith("hello", 0))
print("hello world".startswith("x", 1))

# with start, end
print("hello world".startswith("world", 6, 11))
print("hello world".startswith("wor", 6, 9))

# endswith with indices
print("hello world".endswith("hello", 0, 5))
print("hello world".endswith("llo", 0, 5))

# tuple of prefixes
print("hello".startswith(("hi", "hello", "hey")))
print("hello".startswith(("hi", "hey")))
print("hello".startswith(("hello",)))

# tuple of suffixes
print("filename.py".endswith((".py", ".txt", ".md")))
print("filename.txt".endswith((".py", ".txt", ".md")))
print("filename.exe".endswith((".py", ".txt", ".md")))

# common cases
for word in ["apple", "banana", "avocado", "berry"]:
    if word.startswith("a"):
        print(word)

for f in ["a.py", "b.txt", "c.py", "d.md"]:
    if f.endswith(".py"):
        print(f)
