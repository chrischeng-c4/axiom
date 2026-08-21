for i in range(3):
    print(i)
else:
    print("completed")

for i in range(5):
    if i == 2:
        break
else:
    print("should not print")
print("after break")
