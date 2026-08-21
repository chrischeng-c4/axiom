# List sort variants: sort(reverse=True), sort(key=len), sort(key=lambda)
# List sort variants: sort(reverse=True), sort(key=len), sort(key=lambda)
a = [3, 1, 4, 1, 5]
a.sort()
print(a)
a.sort(reverse=True)
print(a)
b = ['banana', 'apple', 'cherry']
b.sort(key=len)
print(b)
c = [3, 1, 2]
c.sort(key=lambda x: -x)
print(c)
