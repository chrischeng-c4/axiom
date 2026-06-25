class A[X]:
    def b[Y](self):
        class C[Z]:
            pass
        print("Inside b, C:", C)
        return C

print("Creating A instance...")
a = A()
print("Calling a.b()...")
result = a.b()
print("Result:", result)
