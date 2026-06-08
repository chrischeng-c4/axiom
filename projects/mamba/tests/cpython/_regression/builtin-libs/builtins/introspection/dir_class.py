class A:
    def foo(self):
        return 1

class B(A):
    def bar(self):
        return 2

b = B()
b.x = 99
r = dir(b)
print("foo" in r)
print("bar" in r)
print("x" in r)
print(r == sorted(r))
