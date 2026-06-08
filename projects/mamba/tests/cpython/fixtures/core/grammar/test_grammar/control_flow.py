# RUN: parse
# CPython 3.12 test_grammar: control flow

# if / elif / else
if True:
    pass
elif False:
    pass
else:
    pass

# while
while False:
    pass
else:
    pass

# for
for i in range(10):
    pass
else:
    pass

# break / continue
for i in range(10):
    if i == 5:
        break
    if i % 2 == 0:
        continue

# try / except / else / finally
try:
    pass
except ValueError:
    pass
except (TypeError, RuntimeError):
    pass
except Exception as e:
    pass
else:
    pass
finally:
    pass

# try / except*  (ExceptionGroup - Python 3.11+)
try:
    pass
except* ValueError as eg:
    pass
