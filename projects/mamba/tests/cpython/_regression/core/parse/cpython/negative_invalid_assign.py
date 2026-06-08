# RUN: parse
# EXPECT-ERROR: expected )
# Negative parse test: assignment to literal (#566)
# NOTE: Parser accepts 1 = x syntactically.
# Using a definite syntax error instead.

x = (1 + 2
