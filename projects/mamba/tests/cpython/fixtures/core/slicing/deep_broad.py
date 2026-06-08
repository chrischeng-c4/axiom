# slicing operations deep broad

# basic slicing
li = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
print(li[2:5])
print(li[:5])
print(li[5:])
print(li[:])
print(li[::2])
print(li[1::2])
print(li[::-1])

# negative
print(li[-3:])
print(li[:-3])
print(li[-5:-1])

# empty slices
print(li[5:5])
print(li[100:200])
print(li[-100:0])

# single slice
print(li[0:1])
print(li[9:10])

# slice of string
s = "hello world"
print(s[0:5])
print(s[6:])
print(s[::2])
print(s[::-1])
print(s[-5:])
print(s[:-6])

# slice of tuple
t = (0, 1, 2, 3, 4, 5)
print(t[2:5])
print(t[:3])
print(t[3:])
print(t[::2])

# nested slicing
mat = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
print(mat[0])
print(mat[1])
print(mat[0][1:])
print(mat[-1])

# range slicing via list
r = list(range(20))
print(r[5:15])
print(r[::3])
print(r[::-2])

# slice with variable bounds
start = 2
end = 6
print(li[start:end])
print(li[start:])
print(li[:end])

# copy via slice
orig = [1, 2, 3]
copy1 = orig[:]
copy1.append(4)
print(orig)
print(copy1)

# reversed string via slice
s2 = "abc"
print(s2[::-1])

# slicing an empty list
empty = []
print(empty[:])
print(empty[0:5])
print(empty[-10:10])
