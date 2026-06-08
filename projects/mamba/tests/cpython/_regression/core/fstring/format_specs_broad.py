# format specs broad

# int formatting
print("{}".format(42))
print("{:d}".format(42))
print("{:5d}".format(42))
print("{:05d}".format(42))
print("{:<5d}".format(42))
print("{:>5d}".format(42))
print("{:^5d}".format(42))

# float
print("{:.2f}".format(3.14159))
print("{:.0f}".format(3.14159))
print("{:.4f}".format(3.14159))
print("{:10.2f}".format(3.14))
print("{:>10.2f}".format(3.14))
print("{:010.2f}".format(3.14))

# string
print("{}".format("hi"))
print("{:10}".format("hi"))
print("{:>10}".format("hi"))
print("{:<10}".format("hi"))
print("{:^10}".format("hi"))
print("{:.3}".format("hello"))

# positional
print("{0} {1} {0}".format("a", "b"))
print("{1} {0}".format("first", "second"))

# named
print("{name} is {age}".format(name="alice", age=30))

# multiple in one string
print("{} + {} = {}".format(1, 2, 3))

# f-strings
name = "alice"
age = 30
pi = 3.14159
print(f"{name}")
print(f"{name} is {age}")
print(f"pi = {pi:.2f}")
print(f"{age:05d}")
print(f"{age:>5}")
