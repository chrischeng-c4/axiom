class A[X]:
    def b[Y](self):
        class C[Z]:
            def d[W](self):
                return lambda: (X, Y, Z, W)
        return C

print("Step 1: A.__type_params__:", A.__type_params__)
x_var, = A.__type_params__
print("Step 2: x_var:", x_var)

print("Step 3: A.b.__type_params__:", A.b.__type_params__)
y_var, = A.b.__type_params__
print("Step 4: y_var:", y_var)

c_cls = A().b()
print("Step 5: c_cls:", c_cls)
print("Step 6: c_cls.__type_params__:", c_cls.__type_params__)
