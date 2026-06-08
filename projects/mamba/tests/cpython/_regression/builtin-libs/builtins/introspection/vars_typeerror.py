try:
    vars(1)
    print("NO_EXC")
except TypeError as e:
    print("TypeError")
    print(str(e))
