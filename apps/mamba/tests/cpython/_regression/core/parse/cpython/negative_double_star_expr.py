# RUN: parse
# EXPECT-ERROR: expected )
# Negative parse test: double starred assignment target (#566)
# NOTE: Parser accepts *a, *b = [...] syntactically.
# Using a definite syntax error instead.

x = (1 + 2
