# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# string indexing / slicing patterns broad

s = "hello world"

# index
print(s[0])
print(s[1])
print(s[4])
print(s[5])
print(s[10])
print(s[-1])
print(s[-2])
print(s[-11])

# len
print(len(s))
print(len(""))
print(len("a"))
print(len("abcdef"))

# slice basic
print(s[0:5])
print(s[6:])
print(s[:5])
print(s[:])

# slice negative
print(s[-5:])
print(s[:-5])
print(s[-5:-1])

# slice step
print(s[::1])
print(s[::2])
print(s[1::2])
print(s[::-1])
print(s[::3])

# slice empty
print(s[5:5])
print(s[5:3])
print(s[100:])

# slice overshoot
print(s[0:100])
print(s[-100:])
print(s[-100:100])

# compare
print(s[0] == "h")
print(s[-1] == "d")

# string in string
print("hello" in s)
print("world" in s)
print("xyz" in s)
print("" in s)

# char iterate
for c in "abc":
    print(c)

# reversed string
rev = ""
for c in reversed("hello"):
    rev += c
print(rev)

# char freq
def count_char(text, ch):
    count = 0
    for c in text:
        if c == ch:
            count += 1
    return count

print(count_char("abracadabra", "a"))
print(count_char("abracadabra", "b"))
print(count_char("abracadabra", "z"))

# string repetition
print("ab" * 3)
print("-" * 10)
print("" * 5)
print("x" * 0)

# concatenation
a = "hello"
b = "world"
print(a + " " + b)
print(a + b)

# split+slice
parts = s.split()
print(parts[0])
print(parts[1])
print(parts[0][0])
print(parts[1][-1])
