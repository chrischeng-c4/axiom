# string slicing edges broad

s = "abcdefghij"

# basic
print(s[0])
print(s[-1])
print(s[0:3])
print(s[7:])
print(s[:4])
print(s[:])

# negative indexing
print(s[-3])
print(s[-3:])
print(s[:-3])
print(s[-5:-1])

# step
print(s[::2])
print(s[1::2])
print(s[::3])

# reverse
print(s[::-1])

# empty slice
print(s[5:5])
print(s[10:])
print(s[100:])
print(s[-100:])

# single char
print(s[3:4])

# mixed
print(s[1:-1])
print(s[2:-2])

# slice works on tuple (read-only)
t = (1, 2, 3, 4, 5)
print(t[1:4])
print(t[:2])
print(t[2:])

# slice works on bytes
b = b"hello"
print(b[1:3])
print(b[:2])
print(b[2:])
