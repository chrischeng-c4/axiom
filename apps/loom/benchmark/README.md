# loom frontier benchmarks (#127)

Competitor throughput benches for the frontier comparison (results +
analysis: ../docs/benchmark/frontier.md). All run a 500× single `echo`
unit of work, async submit.

- **loom**: `../scripts/bench.sh` (self-contained: boots relay+keep+loom+workers).
- **Celery** (lean task queue): `celery_app.py` + `celery_bench.py`.
  Needs Redis + a venv: `pip install celery 'redis>=4.5,<5'`, then
  `celery -A celery_app worker --concurrency=4 &` and `python celery_bench.py`.
- **Temporal** (durable workflow engine — loom's true peer): `temporal_bench.py`.
  Needs the Temporal dev server (`temporal server start-dev`) +
  `pip install temporalio`, then `python temporal_bench.py`.

Ratchet (CI gate on loom's own throughput): `../scripts/perf-ratchet.sh`
compares to `../docs/benchmark/baseline.json`, fails on regression, ratchets up
on improvement.
