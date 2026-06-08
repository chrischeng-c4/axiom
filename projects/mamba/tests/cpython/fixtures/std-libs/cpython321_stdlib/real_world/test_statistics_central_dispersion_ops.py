# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_statistics_central_dispersion_ops"
# subject = "cpython321.test_statistics_central_dispersion_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_statistics_central_dispersion_ops.py"
# status = "filled"
# ///
"""cpython321.test_statistics_central_dispersion_ops: execute CPython 3.12 seed test_statistics_central_dispersion_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `statistics` standard
# library module — there is no prior `statistics` fixture under
# pass/, only adjacent math/random/decimal seeds. This seed
# asserts: statistics.mean over int and float inputs; mean of
# mixed int/float collapses to float; statistics.fmean produces
# a float regardless of input type; statistics.median (odd-length
# returns middle, even-length returns the average of the two
# middles); statistics.median_low / median_high disambiguate
# even-length medians toward the lower / higher candidate;
# statistics.mode on a unimodal numeric distribution; population
# variance / sample variance distinction
# (pvariance uses N, variance uses N-1); statistics.stdev and
# pstdev are the square roots of variance / pvariance;
# statistics.harmonic_mean (reciprocal-of-mean-of-reciprocals);
# statistics.geometric_mean (n-th root of product); statistics
# robustness over negative-only and zero-centred inputs;
# single-element inputs collapse mean / median / median_low /
# median_high to that element.
import statistics
_ledger: list[int] = []

# mean over int input
assert statistics.mean([1, 2, 3, 4, 5]) == 3; _ledger.append(1)
assert statistics.mean([10, 20, 30]) == 20; _ledger.append(1)
assert statistics.mean([0]) == 0; _ledger.append(1)
assert statistics.mean([42]) == 42; _ledger.append(1)

# mean over float input
assert statistics.mean([1.5, 2.5, 3.5]) == 2.5; _ledger.append(1)
assert statistics.mean([2.0, 4.0, 6.0]) == 4.0; _ledger.append(1)
assert statistics.mean([0.0]) == 0.0; _ledger.append(1)

# mean over mixed int/float — collapses to float
assert statistics.mean([1, 2.0, 3]) == 2.0; _ledger.append(1)
assert statistics.mean([1.0, 2, 3.0, 4]) == 2.5; _ledger.append(1)

# mean over negative-only
assert statistics.mean([-5, -10, -15]) == -10; _ledger.append(1)
assert statistics.mean([-1.0, -2.0, -3.0]) == -2.0; _ledger.append(1)

# mean over zero-centred sequence
assert statistics.mean([-1, 0, 1]) == 0; _ledger.append(1)
assert statistics.mean([-2, -1, 0, 1, 2]) == 0; _ledger.append(1)

# fmean always returns float
assert statistics.fmean([1, 2, 3, 4, 5]) == 3.0; _ledger.append(1)
assert statistics.fmean([2.0, 4.0, 6.0]) == 4.0; _ledger.append(1)
assert statistics.fmean([10]) == 10.0; _ledger.append(1)

# median — odd length returns middle
assert statistics.median([1, 3, 5]) == 3; _ledger.append(1)
assert statistics.median([1, 2, 3, 4, 5]) == 3; _ledger.append(1)
assert statistics.median([42]) == 42; _ledger.append(1)
assert statistics.median([1, 2, 3]) == 2; _ledger.append(1)

# median — even length returns average of the two middles
assert statistics.median([1, 2, 3, 4]) == 2.5; _ledger.append(1)
assert statistics.median([1, 3]) == 2.0; _ledger.append(1)
assert statistics.median([10, 20, 30, 40]) == 25.0; _ledger.append(1)

# median_low — bias toward the lower-middle candidate
assert statistics.median_low([1, 2, 3, 4]) == 2; _ledger.append(1)
assert statistics.median_low([1, 2]) == 1; _ledger.append(1)
assert statistics.median_low([1, 2, 3]) == 2; _ledger.append(1)
assert statistics.median_low([42]) == 42; _ledger.append(1)

# median_high — bias toward the higher-middle candidate
assert statistics.median_high([1, 2, 3, 4]) == 3; _ledger.append(1)
assert statistics.median_high([1, 2]) == 2; _ledger.append(1)
assert statistics.median_high([1, 2, 3]) == 2; _ledger.append(1)
assert statistics.median_high([42]) == 42; _ledger.append(1)

# mode on a unimodal numeric distribution
assert statistics.mode([1, 1, 2, 3]) == 1; _ledger.append(1)
assert statistics.mode([5, 5, 5, 10]) == 5; _ledger.append(1)
assert statistics.mode([1, 2, 2, 3]) == 2; _ledger.append(1)
assert statistics.mode([42]) == 42; _ledger.append(1)

# variance (sample, N-1 denominator)
assert statistics.variance([1, 2, 3, 4, 5]) == 2.5; _ledger.append(1)
assert round(statistics.variance([2, 4, 4, 4, 5, 5, 7, 9]), 6) == 4.571429; _ledger.append(1)

# pvariance (population, N denominator)
assert statistics.pvariance([1, 2, 3, 4, 5]) == 2.0; _ledger.append(1)
assert statistics.pvariance([2, 4, 4, 4, 5, 5, 7, 9]) == 4.0; _ledger.append(1)

# stdev is the sqrt of variance; pstdev is the sqrt of pvariance
assert round(statistics.stdev([1, 2, 3, 4, 5]), 4) == 1.5811; _ledger.append(1)
assert round(statistics.pstdev([1, 2, 3, 4, 5]), 4) == 1.4142; _ledger.append(1)
# variance of a constant sequence is 0 (population)
assert statistics.pstdev([5, 5, 5, 5]) == 0.0; _ledger.append(1)
assert statistics.pvariance([5, 5, 5, 5]) == 0.0; _ledger.append(1)

# harmonic_mean — reciprocal-of-mean-of-reciprocals
assert statistics.harmonic_mean([1, 2, 4]) == 12 / 7; _ledger.append(1)
assert statistics.harmonic_mean([2, 4, 4]) == 3.0; _ledger.append(1)
assert statistics.harmonic_mean([1, 1, 1]) == 1.0; _ledger.append(1)
assert statistics.harmonic_mean([5]) == 5.0; _ledger.append(1)

# geometric_mean — n-th root of product
assert round(statistics.geometric_mean([1, 2, 4]), 6) == 2.0; _ledger.append(1)
assert statistics.geometric_mean([4, 4, 4]) == 4.0; _ledger.append(1)
assert statistics.geometric_mean([1, 1, 1]) == 1.0; _ledger.append(1)
assert round(statistics.geometric_mean([9]), 6) == 9.0; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_statistics_central_dispersion_ops {sum(_ledger)} asserts")
