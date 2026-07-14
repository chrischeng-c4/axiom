#!/usr/bin/env bash
# HANDWRITE-BEGIN gap="missing-generator:logic:125bf141" tracker="#1597" reason="Run an identical simple-protocol pgbench workload through PgBouncer and pgpool."
set -euo pipefail

readonly PROFILE_SCHEMA="pgpool.pgbouncer-baseline.v2"
readonly BACKEND_CAP=16
readonly CLIENTS=64
readonly JOBS=4
readonly DURATION_SECONDS=30
readonly PAIRED_TRIALS=2
readonly MAX_PAIR_RATIO_RELATIVE_SPREAD=0.20
readonly METER_DURATION_CAP_SECONDS=$((DURATION_SECONDS + 30))
readonly SCALE=1
readonly POOL_ACQUIRE_TIMEOUT_MS=60000
readonly DATABASE="pgpool_bench"
readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd)"

PGPOOL_BIN="${PGPOOL_BIN:-$REPO_ROOT/target/release/pgpool}"
METER_BIN=""
DRY_RUN=false
WORKLOAD="tpcb"
WORKLOAD_PROFILE=""
WORK_DIR=""
POSTGRES_STARTED=false
PGBOUNCER_PID=""
PGPOOL_PID=""
USED_PORTS=()
NEXT_FREE_PORT=""
KEEP_WORK_DIR="${PGPOOL_BENCH_KEEP_WORK_DIR:-false}"

usage() {
    cat <<'USAGE'
Usage: run.sh [--dry-run] [--workload tpcb|select-only] [--pgpool-bin PATH] [--meter-bin PATH]

Compares PgBouncer and pgpool transaction pooling against one temporary local
PostgreSQL backend. `--dry-run` prints the immutable profile JSON and does not
inspect the machine, create files, bind ports, or start processes.

`--meter-bin` is an opt-in pgpool-only diagnostic: meter launches and samples
the pgpool process while its opaque driver runs the same pgbench leg. It retains
the temporary work directory with the meter JSON and folded stacks, and labels
the resulting comparison as diagnostic-only rather than win evidence.

`--workload select-only` keeps every pooler input fixed but removes TPC-B's
update-row lock contention, so it measures transaction-pool relay and reset
throughput. The default `tpcb` profile remains unchanged as the database-stress
regression workload.

An ordinary peer comparison runs two 30-second paired trials in opposite
target orders. It prints raw samples and marks a pgpool candidate inconclusive
when the two paired TPS ratios differ by more than 20 percent. A pgpool win
additionally requires both pairs to favor pgpool; two PgBouncer-favoring pairs
reject a candidate even when their loss magnitude varies. `--meter-bin` remains a
single pgpool diagnostic and cannot establish a valid competitor comparison.
USAGE
}

configure_workload() {
    case "$WORKLOAD" in
        tpcb) WORKLOAD_PROFILE="pgbench-tpcb" ;;
        select-only) WORKLOAD_PROFILE="pgbench-select-only" ;;
        *) fail "unknown workload '$WORKLOAD'; use tpcb or select-only" ;;
    esac
}

emit_dry_run_profile() {
    cat <<JSON
{"schema":"$PROFILE_SCHEMA","profile":{"workload":"$WORKLOAD_PROFILE","protocol":"simple","pool_mode":"transaction","backend_connection_cap":$BACKEND_CAP,"clients":$CLIENTS,"jobs":$JOBS,"duration_seconds":$DURATION_SECONDS,"paired_trials":$PAIRED_TRIALS,"orders":["pgbouncer-first","pgpool-first"],"max_pair_ratio_relative_spread":$MAX_PAIR_RATIO_RELATIVE_SPREAD,"scale":$SCALE,"pool_acquire_timeout_ms":$POOL_ACQUIRE_TIMEOUT_MS},"targets":{"pgbouncer":{"pool_mode":"transaction","backend_connection_cap":$BACKEND_CAP,"reset_between_owners":"DISCARD ALL","reset_on_return_to_idle":true},"pgpool":{"pool_mode":"transaction","backend_connection_cap":$BACKEND_CAP,"pool_acquire_timeout_ms":$POOL_ACQUIRE_TIMEOUT_MS,"reset_between_owners":"DISCARD ALL","reset_on_return_to_idle":true}}}
JSON
}

run_pgbench_workload() {
    local host="$1"
    local port="$2"
    local clients="$3"
    local jobs="$4"
    local seconds="$5"
    local output="$6"
    local workload_args=()
    if [[ "$WORKLOAD" == "select-only" ]]; then
        workload_args+=(--select-only)
    fi
    pgbench --no-vacuum --protocol simple "${workload_args[@]}" --client "$clients" --jobs "$jobs" --time "$seconds" --host "$host" --port "$port" --username postgres "$DATABASE" >"$output" 2>&1
}

fail() {
    echo "error: $*" >&2
    exit 1
}

require_command() {
    command -v "$1" >/dev/null 2>&1 || fail "required command '$1' was not found"
}

absolute_executable_path() {
    local path="$1"
    local directory
    directory="$(cd "$(dirname "$path")" && pwd)"
    printf '%s/%s\n' "$directory" "$(basename "$path")"
}

find_free_port() {
    local port
    for port in $(seq 55432 55531); do
        if [[ " ${USED_PORTS[*]-} " == *" $port "* ]]; then
            continue
        fi
        if ! lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1; then
            USED_PORTS+=("$port")
            NEXT_FREE_PORT="$port"
            return 0
        fi
    done
    fail "could not find an unused TCP port in 55432-55531"
}

wait_for_sql() {
    local port="$1"
    local label="$2"
    local _
    for _ in $(seq 1 100); do
        if PGCONNECT_TIMEOUT=1 psql --host 127.0.0.1 --port "$port" --username postgres --dbname "$DATABASE" --no-psqlrc --quiet --command 'SELECT 1' >/dev/null 2>&1; then
            return 0
        fi
        sleep 0.1
    done
    fail "$label did not accept SQL connections; inspect $WORK_DIR"
}

metric() {
    local report="$1"
    local field="$2"
    case "$field" in
        tps) awk '/^tps =/ { print $3; exit }' "$report" ;;
        latency_ms) awk '/^latency average =/ { print $4; exit }' "$report" ;;
        *) fail "unknown metric '$field'" ;;
    esac
}

reported_clients() {
    awk -F: '/^number of clients:/ { gsub(/^[[:space:]]+|[[:space:]]+$/, "", $2); print $2; exit }' "$1"
}

require_metric() {
    local value="$1"
    local name="$2"
    [[ "$value" =~ ^[0-9]+([.][0-9]+)?$ ]] || fail "could not parse $name from pgbench output"
    awk -v value="$value" 'BEGIN { exit !(value > 0) }' || fail "$name must be positive, got '$value'"
}

require_client_count() {
    local value="$1"
    local target="$2"
    [[ "$value" == "$CLIENTS" ]] || fail "$target benchmark did not establish all $CLIENTS clients (reported '$value')"
}

require_clean_pgbench() {
    local report="$1"
    local target="$2"
    if grep -q '^pgbench: error:' "$report"; then
        fail "$target benchmark reported client errors; refusing a partial-workload comparison"
    fi
}

validate_benchmark_sample() {
    local report="$1"
    local target="$2"
    local tps
    local latency_ms
    local clients
    tps="$(metric "$report" tps)"
    latency_ms="$(metric "$report" latency_ms)"
    clients="$(reported_clients "$report")"
    require_metric "$tps" "$target TPS"
    require_metric "$latency_ms" "$target latency"
    require_client_count "$clients" "$target"
    require_clean_pgbench "$report" "$target"
}

mean_two() {
    awk -v first="$1" -v second="$2" 'BEGIN { printf "%.6f", (first + second) / 2 }'
}

ratio() {
    awk -v numerator="$1" -v denominator="$2" 'BEGIN { printf "%.6f", numerator / denominator }'
}

relative_spread() {
    awk -v first="$1" -v second="$2" '
        function abs(value) { return value < 0 ? -value : value }
        BEGIN {
            mean = (first + second) / 2
            printf "%.6f", mean == 0 ? 1 : abs(first - second) / mean
        }
    '
}

pair_winner() {
    awk -v value="$1" 'BEGIN { print (value > 1 ? "pgpool" : (value < 1 ? "pgbouncer" : "tie")) }'
}

write_meter_driver() {
    cat >"$WORK_DIR/pgpool-meter-drive.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -euo pipefail

for _ in $(seq 1 100); do
    if PGCONNECT_TIMEOUT=1 psql --host "$PGPOOL_BENCH_PGPOOL_HOST" --port "$PGPOOL_BENCH_PGPOOL_PORT" --username postgres --dbname "$PGPOOL_BENCH_DATABASE" --no-psqlrc --quiet --command 'SELECT 1' >/dev/null 2>&1; then
        workload_args=()
        if [[ "$PGPOOL_BENCH_WORKLOAD" == "select-only" ]]; then
            workload_args+=(--select-only)
        fi
        exec pgbench --no-vacuum --protocol simple "${workload_args[@]}" --client "$PGPOOL_BENCH_CLIENTS" --jobs "$PGPOOL_BENCH_JOBS" --time "$PGPOOL_BENCH_DURATION_SECONDS" --host "$PGPOOL_BENCH_PGPOOL_HOST" --port "$PGPOOL_BENCH_PGPOOL_PORT" --username postgres "$PGPOOL_BENCH_DATABASE" >"$PGPOOL_BENCH_PGPOOL_PGBENCH_LOG" 2>&1
    fi
    sleep 0.1
done

echo "pgpool did not accept SQL connections for meter diagnostic" >&2
exit 1
SCRIPT
    chmod +x "$WORK_DIR/pgpool-meter-drive.sh"
}

require_meter_artifacts() {
    local collapsed=()
    [[ -s "$WORK_DIR/meter-report.json" ]] || fail "meter did not produce its JSON report"
    [[ -s "$WORK_DIR/.meter/last-report.json" ]] || fail "meter did not persist .meter/last-report.json"
    grep -q '"schema_version": "meter.report/1"' "$WORK_DIR/meter-report.json" || fail "meter report did not use meter.report/1"
    shopt -s nullglob
    collapsed=("$WORK_DIR/.meter/"*.collapsed)
    shopt -u nullglob
    ((${#collapsed[@]} > 0)) || fail "meter did not produce collapsed stack evidence"
}

start_pgpool() {
    "$PGPOOL_BIN" serve \
        --backend-host 127.0.0.1 \
        --backend-port "$POSTGRES_PORT" \
        --bind "127.0.0.1:$PGPOOL_PORT" \
        --admin-bind "127.0.0.1:$ADMIN_PORT" \
        --max-backend-connections "$BACKEND_CAP" \
        --pool-acquire-timeout-ms "$POOL_ACQUIRE_TIMEOUT_MS" \
        >"$WORK_DIR/pgpool.log" 2>&1 &
    PGPOOL_PID=$!
    wait_for_sql "$PGPOOL_PORT" "pgpool"
}

run_pgpool_meter_diagnostic() {
    [[ -n "$METER_BIN" ]] || fail "meter diagnostic requires --meter-bin"
    write_meter_driver
    KEEP_WORK_DIR=true
    (
        cd "$WORK_DIR"
        PGPOOL_BENCH_PGPOOL_HOST=127.0.0.1 \
        PGPOOL_BENCH_PGPOOL_PORT="$PGPOOL_PORT" \
        PGPOOL_BENCH_DATABASE="$DATABASE" \
        PGPOOL_BENCH_CLIENTS="$CLIENTS" \
        PGPOOL_BENCH_JOBS="$JOBS" \
        PGPOOL_BENCH_WORKLOAD="$WORKLOAD" \
        PGPOOL_BENCH_DURATION_SECONDS="$DURATION_SECONDS" \
        PGPOOL_BENCH_PGPOOL_PGBENCH_LOG="$WORK_DIR/pgpool-pgbench.log" \
        "$METER_BIN" measure "$PGPOOL_BIN" --level sample --duration-cap "$METER_DURATION_CAP_SECONDS" --drive ./pgpool-meter-drive.sh -- serve \
            --backend-host 127.0.0.1 \
            --backend-port "$POSTGRES_PORT" \
            --bind "127.0.0.1:$PGPOOL_PORT" \
            --admin-bind "127.0.0.1:$ADMIN_PORT" \
            --max-backend-connections "$BACKEND_CAP" \
            --pool-acquire-timeout-ms "$POOL_ACQUIRE_TIMEOUT_MS" \
            >"$WORK_DIR/meter-report.json" 2>"$WORK_DIR/meter.log"
    )
    require_meter_artifacts
    echo "meter sampled pgpool only; its comparison result is diagnostic-only, not win evidence" >&2
}

cleanup() {
    set +e
    if [[ -n "$PGPOOL_PID" ]]; then
        kill "$PGPOOL_PID" 2>/dev/null
        wait "$PGPOOL_PID" 2>/dev/null
    fi
    if [[ -n "$PGBOUNCER_PID" ]]; then
        kill "$PGBOUNCER_PID" 2>/dev/null
        wait "$PGBOUNCER_PID" 2>/dev/null
    fi
    if [[ "$POSTGRES_STARTED" == true ]]; then
        pg_ctl --pgdata "$WORK_DIR/postgres" --wait --mode fast stop >/dev/null 2>&1
    fi
    if [[ -n "$WORK_DIR" && "$KEEP_WORK_DIR" == true ]]; then
        echo "benchmark artifacts retained at $WORK_DIR" >&2
    elif [[ -n "$WORK_DIR" ]]; then
        rm -rf "$WORK_DIR"
    fi
}

while (($#)); do
    case "$1" in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --pgpool-bin)
            (($# >= 2)) || fail "--pgpool-bin requires a path"
            PGPOOL_BIN="$2"
            shift 2
            ;;
        --meter-bin)
            (($# >= 2)) || fail "--meter-bin requires a path"
            METER_BIN="$2"
            shift 2
            ;;
        --workload)
            (($# >= 2)) || fail "--workload requires tpcb or select-only"
            WORKLOAD="$2"
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            fail "unknown argument '$1'; use --help"
            ;;
    esac
done

configure_workload

if [[ "$DRY_RUN" == true ]]; then
    emit_dry_run_profile
    exit 0
fi

for command in initdb pg_ctl psql pgbench pgbouncer lsof; do
    require_command "$command"
done
[[ -x "$PGPOOL_BIN" ]] || fail "pgpool binary is not executable: $PGPOOL_BIN (build with: cargo build --release -p pgpool)"
PGPOOL_BIN="$(absolute_executable_path "$PGPOOL_BIN")"
if [[ -n "$METER_BIN" ]]; then
    [[ -x "$METER_BIN" ]] || fail "meter binary is not executable: $METER_BIN (build with: cargo build -p meter-cli --bin meter)"
    METER_BIN="$(absolute_executable_path "$METER_BIN")"
fi

WORK_DIR="$(mktemp -d "${TMPDIR:-/tmp}/pgpool-pgbouncer-benchmark.XXXXXX")"
trap cleanup EXIT INT TERM

find_free_port
POSTGRES_PORT="$NEXT_FREE_PORT"
find_free_port
PGBOUNCER_PORT="$NEXT_FREE_PORT"
find_free_port
PGPOOL_PORT="$NEXT_FREE_PORT"
find_free_port
ADMIN_PORT="$NEXT_FREE_PORT"

initdb --pgdata "$WORK_DIR/postgres" --auth trust --username postgres --no-locale >"$WORK_DIR/initdb.log" 2>&1
pg_ctl --pgdata "$WORK_DIR/postgres" --options "-h 127.0.0.1 -p $POSTGRES_PORT" --wait start >"$WORK_DIR/postgres.log" 2>&1
POSTGRES_STARTED=true

psql --host 127.0.0.1 --port "$POSTGRES_PORT" --username postgres --dbname postgres --no-psqlrc --quiet --command "CREATE DATABASE $DATABASE" >"$WORK_DIR/create-db.log"
pgbench --initialize --scale "$SCALE" --host 127.0.0.1 --port "$POSTGRES_PORT" --username postgres "$DATABASE" >"$WORK_DIR/pgbench-init.log" 2>&1

# PgBouncer 1.25 still requires a recognized login role when auth_type=trust.
# The ephemeral PostgreSQL cluster itself is trust-authenticated, so the
# temporary userlist deliberately contains no reusable credential material.
printf '"postgres" ""\n' >"$WORK_DIR/userlist.txt"

# Warm the identical selected workload before either target is measured.
run_pgbench_workload 127.0.0.1 "$POSTGRES_PORT" 8 2 3 "$WORK_DIR/warmup.log"

cat >"$WORK_DIR/pgbouncer.ini" <<CONFIG
[databases]
$DATABASE = host=127.0.0.1 port=$POSTGRES_PORT dbname=$DATABASE

[pgbouncer]
listen_addr = 127.0.0.1
listen_port = $PGBOUNCER_PORT
auth_type = trust
auth_file = $WORK_DIR/userlist.txt
pool_mode = transaction
max_client_conn = 1000
default_pool_size = $BACKEND_CAP
server_reset_query = DISCARD ALL
server_reset_query_always = 1
ignore_startup_parameters = extra_float_digits
admin_users = postgres
pidfile = $WORK_DIR/pgbouncer.pid
logfile = $WORK_DIR/pgbouncer.log
CONFIG

pgbouncer "$WORK_DIR/pgbouncer.ini" >"$WORK_DIR/pgbouncer.stdout" 2>&1 &
PGBOUNCER_PID=$!
wait_for_sql "$PGBOUNCER_PORT" "PgBouncer"

if [[ -n "$METER_BIN" ]]; then
    run_pgbench_workload 127.0.0.1 "$PGBOUNCER_PORT" "$CLIENTS" "$JOBS" "$DURATION_SECONDS" "$WORK_DIR/pgbouncer-pgbench.log"
    run_pgpool_meter_diagnostic
    validate_benchmark_sample "$WORK_DIR/pgbouncer-pgbench.log" "PgBouncer"
    validate_benchmark_sample "$WORK_DIR/pgpool-pgbench.log" "pgpool"
    PGBOUNCER_TPS="$(metric "$WORK_DIR/pgbouncer-pgbench.log" tps)"
    PGBOUNCER_LATENCY_MS="$(metric "$WORK_DIR/pgbouncer-pgbench.log" latency_ms)"
    PGPOOL_TPS="$(metric "$WORK_DIR/pgpool-pgbench.log" tps)"
    PGPOOL_LATENCY_MS="$(metric "$WORK_DIR/pgpool-pgbench.log" latency_ms)"
    TPS_RATIO="$(ratio "$PGPOOL_TPS" "$PGBOUNCER_TPS")"
    printf '{"schema":"%s","profile":{"workload":"%s","protocol":"simple","pool_mode":"transaction","backend_connection_cap":%s,"clients":%s,"jobs":%s,"duration_seconds":%s,"paired_trials":%s,"orders":["pgbouncer-first","pgpool-first"],"max_pair_ratio_relative_spread":%s,"scale":%s},"targets":{"pgbouncer":{"tps":%s,"latency_average_ms":%s},"pgpool":{"tps":%s,"latency_average_ms":%s}},"ratios":{"pgpool_over_pgbouncer_tps":%s},"comparison_valid":false,"pgpool_win_eligible":false,"winner_by_tps":"diagnostic-only","diagnostics":{"meter_sampled_pgpool":true,"comparison_valid":false}}\n' \
        "$PROFILE_SCHEMA" "$WORKLOAD_PROFILE" "$BACKEND_CAP" "$CLIENTS" "$JOBS" "$DURATION_SECONDS" "$PAIRED_TRIALS" "$MAX_PAIR_RATIO_RELATIVE_SPREAD" "$SCALE" \
        "$PGBOUNCER_TPS" "$PGBOUNCER_LATENCY_MS" "$PGPOOL_TPS" "$PGPOOL_LATENCY_MS" "$TPS_RATIO"
    exit 0
fi

# Both poolers are ready before measurement.  The first pair gives PgBouncer
# first position; the second gives pgpool first position, without overlapping
# pgbench traffic on the shared capped backend.
start_pgpool

PGBOUNCER_FIRST_LOG="$WORK_DIR/pgbouncer-first-pgbench.log"
PGPOOL_SECOND_LOG="$WORK_DIR/pgpool-second-pgbench.log"
PGPOOL_FIRST_LOG="$WORK_DIR/pgpool-first-pgbench.log"
PGBOUNCER_SECOND_LOG="$WORK_DIR/pgbouncer-second-pgbench.log"

run_pgbench_workload 127.0.0.1 "$PGBOUNCER_PORT" "$CLIENTS" "$JOBS" "$DURATION_SECONDS" "$PGBOUNCER_FIRST_LOG"
validate_benchmark_sample "$PGBOUNCER_FIRST_LOG" "PgBouncer first trial"
run_pgbench_workload 127.0.0.1 "$PGPOOL_PORT" "$CLIENTS" "$JOBS" "$DURATION_SECONDS" "$PGPOOL_SECOND_LOG"
validate_benchmark_sample "$PGPOOL_SECOND_LOG" "pgpool second trial"
run_pgbench_workload 127.0.0.1 "$PGPOOL_PORT" "$CLIENTS" "$JOBS" "$DURATION_SECONDS" "$PGPOOL_FIRST_LOG"
validate_benchmark_sample "$PGPOOL_FIRST_LOG" "pgpool first trial"
run_pgbench_workload 127.0.0.1 "$PGBOUNCER_PORT" "$CLIENTS" "$JOBS" "$DURATION_SECONDS" "$PGBOUNCER_SECOND_LOG"
validate_benchmark_sample "$PGBOUNCER_SECOND_LOG" "PgBouncer second trial"

PGBOUNCER_FIRST_TPS="$(metric "$PGBOUNCER_FIRST_LOG" tps)"
PGBOUNCER_FIRST_LATENCY_MS="$(metric "$PGBOUNCER_FIRST_LOG" latency_ms)"
PGPOOL_SECOND_TPS="$(metric "$PGPOOL_SECOND_LOG" tps)"
PGPOOL_SECOND_LATENCY_MS="$(metric "$PGPOOL_SECOND_LOG" latency_ms)"
PGPOOL_FIRST_TPS="$(metric "$PGPOOL_FIRST_LOG" tps)"
PGPOOL_FIRST_LATENCY_MS="$(metric "$PGPOOL_FIRST_LOG" latency_ms)"
PGBOUNCER_SECOND_TPS="$(metric "$PGBOUNCER_SECOND_LOG" tps)"
PGBOUNCER_SECOND_LATENCY_MS="$(metric "$PGBOUNCER_SECOND_LOG" latency_ms)"

FIRST_RATIO="$(ratio "$PGPOOL_SECOND_TPS" "$PGBOUNCER_FIRST_TPS")"
SECOND_RATIO="$(ratio "$PGPOOL_FIRST_TPS" "$PGBOUNCER_SECOND_TPS")"
TPS_RATIO="$(mean_two "$FIRST_RATIO" "$SECOND_RATIO")"
PAIR_RATIO_RELATIVE_SPREAD="$(relative_spread "$FIRST_RATIO" "$SECOND_RATIO")"
FIRST_PAIR_WINNER="$(pair_winner "$FIRST_RATIO")"
SECOND_PAIR_WINNER="$(pair_winner "$SECOND_RATIO")"
if [[ "$FIRST_PAIR_WINNER" == "$SECOND_PAIR_WINNER" ]]; then
    UNANIMOUS_DIRECTION="$FIRST_PAIR_WINNER"
else
    UNANIMOUS_DIRECTION="mixed"
fi
STABLE="$(awk -v spread="$PAIR_RATIO_RELATIVE_SPREAD" -v limit="$MAX_PAIR_RATIO_RELATIVE_SPREAD" 'BEGIN { print (spread <= limit ? "true" : "false") }')"
if [[ "$UNANIMOUS_DIRECTION" == "pgpool" && "$STABLE" == true ]]; then
    PGPOOL_WIN_ELIGIBLE=true
else
    PGPOOL_WIN_ELIGIBLE=false
fi
if [[ "$UNANIMOUS_DIRECTION" == "pgbouncer" || "$PGPOOL_WIN_ELIGIBLE" == true ]]; then
    COMPARISON_VALID=true
    WINNER="$UNANIMOUS_DIRECTION"
else
    COMPARISON_VALID=false
    WINNER="invalid"
fi
PGBOUNCER_TPS="$(mean_two "$PGBOUNCER_FIRST_TPS" "$PGBOUNCER_SECOND_TPS")"
PGBOUNCER_LATENCY_MS="$(mean_two "$PGBOUNCER_FIRST_LATENCY_MS" "$PGBOUNCER_SECOND_LATENCY_MS")"
PGPOOL_TPS="$(mean_two "$PGPOOL_FIRST_TPS" "$PGPOOL_SECOND_TPS")"
PGPOOL_LATENCY_MS="$(mean_two "$PGPOOL_FIRST_LATENCY_MS" "$PGPOOL_SECOND_LATENCY_MS")"

printf '{"schema":"%s","profile":{"workload":"%s","protocol":"simple","pool_mode":"transaction","backend_connection_cap":%s,"clients":%s,"jobs":%s,"duration_seconds":%s,"paired_trials":%s,"orders":["pgbouncer-first","pgpool-first"],"max_pair_ratio_relative_spread":%s,"scale":%s},"trials":[{"order":"pgbouncer-first","targets":{"pgbouncer":{"tps":%s,"latency_average_ms":%s},"pgpool":{"tps":%s,"latency_average_ms":%s}},"ratios":{"pgpool_over_pgbouncer_tps":%s},"winner_by_tps":"%s"},{"order":"pgpool-first","targets":{"pgbouncer":{"tps":%s,"latency_average_ms":%s},"pgpool":{"tps":%s,"latency_average_ms":%s}},"ratios":{"pgpool_over_pgbouncer_tps":%s},"winner_by_tps":"%s"}],"targets":{"pgbouncer":{"tps":%s,"latency_average_ms":%s},"pgpool":{"tps":%s,"latency_average_ms":%s}},"ratios":{"pgpool_over_pgbouncer_tps":%s,"pgpool_over_pgbouncer_tps_first_pair":%s,"pgpool_over_pgbouncer_tps_second_pair":%s,"pair_ratio_relative_spread":%s,"stable":%s,"unanimous_direction":"%s"},"comparison_valid":%s,"pgpool_win_eligible":%s,"winner_by_tps":"%s"}\n' \
    "$PROFILE_SCHEMA" "$WORKLOAD_PROFILE" "$BACKEND_CAP" "$CLIENTS" "$JOBS" "$DURATION_SECONDS" "$PAIRED_TRIALS" "$MAX_PAIR_RATIO_RELATIVE_SPREAD" "$SCALE" \
    "$PGBOUNCER_FIRST_TPS" "$PGBOUNCER_FIRST_LATENCY_MS" "$PGPOOL_SECOND_TPS" "$PGPOOL_SECOND_LATENCY_MS" "$FIRST_RATIO" "$FIRST_PAIR_WINNER" \
    "$PGBOUNCER_SECOND_TPS" "$PGBOUNCER_SECOND_LATENCY_MS" "$PGPOOL_FIRST_TPS" "$PGPOOL_FIRST_LATENCY_MS" "$SECOND_RATIO" "$SECOND_PAIR_WINNER" \
    "$PGBOUNCER_TPS" "$PGBOUNCER_LATENCY_MS" "$PGPOOL_TPS" "$PGPOOL_LATENCY_MS" "$TPS_RATIO" "$FIRST_RATIO" "$SECOND_RATIO" "$PAIR_RATIO_RELATIVE_SPREAD" "$STABLE" "$UNANIMOUS_DIRECTION" "$COMPARISON_VALID" "$PGPOOL_WIN_ELIGIBLE" "$WINNER"
# HANDWRITE-END
