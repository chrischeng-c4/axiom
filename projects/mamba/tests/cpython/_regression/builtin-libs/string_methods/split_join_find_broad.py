# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string split/join/replace/find broad

# split
print("a b c".split())
print("a,b,c".split(","))
print("a  b  c".split())
print("a--b--c".split("--"))
print("abc".split())
print("".split())
print("a".split())
print("one two three four".split(" ", 2))

# splitlines
print("a\nb\nc".splitlines())
print("a\nb\n".splitlines())
print("a\r\nb".splitlines())
print("".splitlines())

# join
print(",".join(["a", "b", "c"]))
print("".join(["a", "b", "c"]))
print(" ".join(["hello", "world"]))
print("-".join([]))
print("-".join(["solo"]))
print("-".join(["a"]))
print("--".join(["a", "b"]))

# join over string
print("-".join("abc"))
print(" ".join("xy"))

# replace
print("hello".replace("l", "L"))
print("hello".replace("ll", "LL"))
print("aaaa".replace("a", "b"))
print("aaaa".replace("a", "b", 2))
print("hello".replace("xyz", "abc"))
print("".replace("a", "b"))
print("abcabc".replace("abc", ""))

# find
print("hello world".find("world"))
print("hello world".find("xyz"))
print("hello".find("ll"))
print("hello".find("o"))
print("hello".find("h"))
print("".find("x"))
print("hello".find(""))

# rfind
print("hello world".rfind("o"))
print("hello world".rfind("l"))
print("hello".rfind("xyz"))

# index (exceptions removed, only found)
print("hello".index("l"))
print("abcabc".index("c"))

# count
print("hello".count("l"))
print("hello".count("x"))
print("abababab".count("ab"))
print("abababab".count("aba"))
print("".count("x"))

# partition / rpartition
print("key=value".partition("="))
print("key=value=extra".partition("="))
print("key=value=extra".rpartition("="))
print("no-sep".partition("x"))

# concat / multiply
print("ab" + "cd")
print("ab" * 3)
print("x" * 5)
print("" * 10)

# compare
print("abc" == "abc")
print("abc" < "abd")
print("abc" < "abcd")
print("Abc" < "abc")
