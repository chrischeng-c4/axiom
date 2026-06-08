# RUN: parse
# EXPECT-ERROR: expected identifier
# Negative parse test: invalid class definition (#566)

class 456:
    pass
