# RUN: parse
# EXPECT-ERROR: expected identifier
# Negative parse test: invalid function definition (#566)

def 123():
    pass
