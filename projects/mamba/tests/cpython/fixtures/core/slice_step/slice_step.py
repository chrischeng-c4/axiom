print([0,1,2,3,4][::2])
print([0,1,2,3,4][::-1])
print("hello"[::2])
try:
    x = [1,2,3][::0]
except ValueError:
    print("step zero ok")
