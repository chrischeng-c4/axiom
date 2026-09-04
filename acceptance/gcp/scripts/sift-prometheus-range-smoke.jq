.status == "success"
and .data.resultType == "matrix"
and (.data.result | length) == 1
and .data.result[0].metric.__name__ == "sift_acceptance_total"
and .data.result[0].metric.fixture == "smoke-remote-write"
and .data.result[0].values == [
  [$epoch, "0"],
  [$epoch + 1, "1"],
  [$epoch + 2, "1"]
]
