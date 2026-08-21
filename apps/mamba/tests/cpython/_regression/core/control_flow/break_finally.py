# break inside try/finally — finally block runs on break
for i in range(5):
    try:
        if i == 2:
            break
        print(i)
    finally:
        print("finally", i)
