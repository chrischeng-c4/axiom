# <HANDWRITE gap="missing-generator:logic:c2ccca76" tracker="pending-tracker" reason="scaffold for apps/beam/benchmark/competitor_bench.py — fill in by hand and update tracker when codegen is ready">
#!/usr/bin/env python3
"""Same-machine CPU competitor baseline for beam's GPU vector search.

This is the CPU side of `competitor-performance-baseline.md`: a real, exact L2
kNN competitor measured on the *same Mac* as `beam bench`, so the comparison is
wall-clock query latency + throughput on identical dataset shapes (dim=128,
k=10, 200 queries, n in {10k, 100k, 1M}).

Competitor preference order (auto-detected, prints which was used):
  1. faiss-cpu  -> IndexFlatL2 (exact) + IndexIVFPQ (ANN).  Gold-standard ANN lib.
  2. hnswlib    -> brute-force space (exact) as a flat fallback.
  3. numpy      -> plain brute-force L2 kNN (always available).

Distribution matches `beam bench --index flat`: components uniform in [-1, 1),
seeded, so the run is deterministic. beam builds its own (identically shaped)
uniform corpus internally, so both sides are exact and recall@10 = 1.000 by
construction; the measured quantity is latency + throughput.

Methodology (matched to `beam bench`, which loops one query at a time on the
GPU, no batching):
  * per_query_ms  = single-query latency, averaged over `queries` searches run
                    ONE AT A TIME (the metric directly comparable to beam's
                    "GPU query timing: avg X ms/query"). One warmup query is
                    discarded first.
  * per_query_qps = 1000 / per_query_ms (single-stream throughput).
  * batched_qps   = throughput when all `queries` are searched in ONE faiss
                    call (faiss's optimal multi-threaded/BLAS batch mode). beam
                    does not batch queries, so this is reported as faiss's best
                    case, NOT the head-to-head number.

faiss uses all OpenMP threads by default (count printed) -- i.e. the strongest
CPU competitor this machine can field, which is the fair bar for "beam-GPU beats
the CPU competitor on this Mac".

Run:
    python3 -m venv /tmp/beambench && /tmp/beambench/bin/pip install faiss-cpu numpy
    /tmp/beambench/bin/python apps/beam/benchmark/competitor_bench.py
    # smaller/faster sweep:
    /tmp/beambench/bin/python apps/beam/benchmark/competitor_bench.py --sizes 10000,100000
"""
from __future__ import annotations

import argparse
import json
import sys
import time

import numpy as np

# --- fixed, deterministic parameters (mirror `beam bench` defaults) ---------
SEED = 1234
DIM = 128
K = 10
QUERIES = 200
# IVF-PQ knobs matched to `beam bench --index ivfpq` defaults.
NLIST = 256
NPROBE = 16
PQ_M = 16
PQ_NBITS = 8
# Faiss-style training-sample cap (beam caps k-means training at 100k too).
TRAIN_SAMPLE_CAP = 100_000


def make_data(n: int, dim: int, queries: int, seed: int):
    """Seeded uniform [-1, 1) corpus + queries, matching beam's flat dataset."""
    rng = np.random.default_rng(seed)
    xb = (rng.random((n, dim), dtype=np.float32) * 2.0 - 1.0).astype(np.float32)
    xq = (rng.random((queries, dim), dtype=np.float32) * 2.0 - 1.0).astype(np.float32)
    return np.ascontiguousarray(xb), np.ascontiguousarray(xq)


def recall_at_k(approx_ids: np.ndarray, truth_ids: np.ndarray) -> float:
    """Mean fraction of each query's true top-k that the approx result found."""
    hits = 0
    total = 0
    for a_row, t_row in zip(approx_ids, truth_ids):
        t = set(int(x) for x in t_row)
        hits += sum(1 for x in a_row if int(x) in t)
        total += len(t_row)
    return hits / total if total else 1.0


def time_per_query(search_one, xq: np.ndarray) -> float:
    """Avg single-query latency (ms) over xq, one query at a time (beam-matched).

    `search_one(q2d)` takes a (1, dim) array and returns (D, I). One warmup
    query is run and discarded before timing.
    """
    search_one(xq[0:1])  # warmup: excludes any one-time setup
    t0 = time.perf_counter()
    for i in range(xq.shape[0]):
        search_one(xq[i : i + 1])
    elapsed = time.perf_counter() - t0
    return elapsed * 1000.0 / xq.shape[0]


def run_faiss(faiss, n: int, xb: np.ndarray, xq: np.ndarray) -> list[dict]:
    rows: list[dict] = []
    dim = xb.shape[1]

    # ---- competitor-CPU-flat: exact IndexFlatL2 (this IS the ground truth) ----
    t0 = time.perf_counter()
    flat = faiss.IndexFlatL2(dim)
    flat.add(xb)
    build_s = time.perf_counter() - t0

    per_ms = time_per_query(lambda q: flat.search(q, K), xq)

    tb0 = time.perf_counter()
    truth_d, truth_i = flat.search(xq, K)  # batched (faiss optimal mode)
    batched_s = time.perf_counter() - tb0
    batched_qps = xq.shape[0] / batched_s if batched_s > 0 else float("inf")

    rows.append(
        {
            "system": "competitor-CPU-flat (faiss IndexFlatL2)",
            "n": n,
            "build_s": build_s,
            "per_query_ms": per_ms,
            "per_query_qps": 1000.0 / per_ms,
            "batched_qps": batched_qps,
            "recall_at_k": 1.000,
        }
    )

    # ---- competitor-CPU-IVFPQ: ANN point (recall vs the exact ground truth) ----
    quantizer = faiss.IndexFlatL2(dim)
    ivfpq = faiss.IndexIVFPQ(quantizer, dim, NLIST, PQ_M, PQ_NBITS)
    train = xb[: min(n, TRAIN_SAMPLE_CAP)]
    t0 = time.perf_counter()
    ivfpq.train(train)
    ivfpq.add(xb)
    build_s = time.perf_counter() - t0
    ivfpq.nprobe = NPROBE

    per_ms = time_per_query(lambda q: ivfpq.search(q, K), xq)

    tb0 = time.perf_counter()
    _, approx_i = ivfpq.search(xq, K)
    batched_s = time.perf_counter() - tb0
    batched_qps = xq.shape[0] / batched_s if batched_s > 0 else float("inf")
    recall = recall_at_k(approx_i, truth_i)

    rows.append(
        {
            "system": f"competitor-CPU-IVFPQ (faiss, nlist={NLIST} m={PQ_M} nprobe={NPROBE})",
            "n": n,
            "build_s": build_s,
            "per_query_ms": per_ms,
            "per_query_qps": 1000.0 / per_ms,
            "batched_qps": batched_qps,
            "recall_at_k": recall,
        }
    )
    return rows


def run_numpy(n: int, xb: np.ndarray, xq: np.ndarray) -> list[dict]:
    """Plain brute-force exact L2 kNN fallback (no faiss/hnswlib)."""
    dim = xb.shape[1]
    # "build" for brute force = precompute squared norms of the corpus.
    t0 = time.perf_counter()
    xb_sq = np.einsum("ij,ij->i", xb, xb)
    build_s = time.perf_counter() - t0

    def search_one(q2d: np.ndarray):
        # ||x - q||^2 = ||x||^2 - 2 x.q + ||q||^2 ; argpartition for top-k.
        d = xb_sq - 2.0 * (xb @ q2d[0])
        idx = np.argpartition(d, K)[:K]
        return None, idx[np.argsort(d[idx])][None, :]

    per_ms = time_per_query(search_one, xq)

    tb0 = time.perf_counter()
    # batched: full (queries x n) distance matrix, then top-k per row.
    dmat = xb_sq[None, :] - 2.0 * (xq @ xb.T)
    part = np.argpartition(dmat, K, axis=1)[:, :K]
    batched_s = time.perf_counter() - tb0
    batched_qps = xq.shape[0] / batched_s if batched_s > 0 else float("inf")

    return [
        {
            "system": "competitor-CPU-flat (numpy brute force)",
            "n": n,
            "build_s": build_s,
            "per_query_ms": per_ms,
            "per_query_qps": 1000.0 / per_ms,
            "batched_qps": batched_qps,
            "recall_at_k": 1.000,
        }
    ]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--sizes", default="10000,100000,1000000",
                    help="comma-separated n values (default 10k,100k,1M)")
    ap.add_argument("--dim", type=int, default=DIM)
    ap.add_argument("--k", type=int, default=K)
    ap.add_argument("--queries", type=int, default=QUERIES)
    ap.add_argument("--json", action="store_true", help="emit JSON only")
    args = ap.parse_args()

    # Update module-level params from CLI (globals() assignment avoids the
    # "used prior to global declaration" rule, since DIM is read in argparse
    # defaults above).
    globals()["DIM"], globals()["K"], globals()["QUERIES"] = args.dim, args.k, args.queries
    sizes = [int(s) for s in args.sizes.split(",") if s.strip()]

    backend = None
    faiss = None
    try:
        import faiss as _faiss  # type: ignore
        faiss = _faiss
        backend = "faiss"
    except Exception:
        try:
            import hnswlib  # type: ignore  # noqa: F401
            backend = "hnswlib"  # (brute-force space fallback not used unless faiss absent)
        except Exception:
            backend = "numpy"

    threads = faiss.omp_get_max_threads() if backend == "faiss" else None
    meta = {
        "competitor_backend": backend,
        "faiss_version": getattr(faiss, "__version__", None) if faiss else None,
        "numpy_version": np.__version__,
        "omp_threads": threads,
        "dim": DIM, "k": K, "queries": QUERIES, "seed": SEED,
        "python": sys.version.split()[0],
    }

    all_rows: list[dict] = []
    for n in sizes:
        xb, xq = make_data(n, DIM, QUERIES, SEED)
        if backend == "faiss":
            all_rows += run_faiss(faiss, n, xb, xq)
        else:
            # hnswlib brute-force and numpy both reduce to the exact numpy path
            # here (kept simple + always-available); note the backend in meta.
            all_rows += run_numpy(n, xb, xq)

    if args.json:
        print(json.dumps({"meta": meta, "rows": all_rows}, indent=2))
        return 0

    print(f"# competitor backend: {backend}"
          + (f" v{meta['faiss_version']} ({threads} OMP threads)" if backend == "faiss" else "")
          + f" | numpy {np.__version__} | dim={DIM} k={K} queries={QUERIES} seed={SEED}")
    hdr = f"{'system':52} {'n':>9} {'build_s':>9} {'q_ms':>9} {'q/s':>10} {'batch_q/s':>11} {'recall':>7}"
    print(hdr)
    print("-" * len(hdr))
    for r in all_rows:
        print(f"{r['system']:52} {r['n']:>9} {r['build_s']:>9.3f} "
              f"{r['per_query_ms']:>9.3f} {r['per_query_qps']:>10.1f} "
              f"{r['batched_qps']:>11.1f} {r['recall_at_k']:>7.3f}")
    print("\n# JSON:")
    print(json.dumps({"meta": meta, "rows": all_rows}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

# </HANDWRITE>
