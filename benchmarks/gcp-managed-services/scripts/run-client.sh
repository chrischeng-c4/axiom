#!/usr/bin/env bash
set -euo pipefail

: "${PROJECT_ID:?PROJECT_ID is required}"
: "${REGION:?REGION is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${CLIENT_IMAGE:?CLIENT_IMAGE is required}"
: "${BENCHMARK_SERVICE_ACCOUNT:?BENCHMARK_SERVICE_ACCOUNT is required}"
: "${RECEIVER_URL:?RECEIVER_URL is required}"
: "${RECEIVER_SECRET:?RECEIVER_SECRET is required}"
: "${PUBSUB_TOPIC:?PUBSUB_TOPIC is required}"
: "${PUBSUB_SUBSCRIPTIONS:?PUBSUB_SUBSCRIPTIONS is required}"
: "${CLOUD_TASKS_QUEUE:?CLOUD_TASKS_QUEUE is required}"

TAPE_READY_REPLICAS="$(kubectl -n tape get statefulset tape -o jsonpath='{.status.readyReplicas}')"
DEFER_READY_REPLICAS="$(kubectl -n defer get statefulset defer -o jsonpath='{.status.readyReplicas}')"
RELAY_READY_REPLICAS="$(kubectl -n relay get statefulset relay -o jsonpath='{.status.readyReplicas}')"

kubectl create namespace axiom-bench --dry-run=client -o yaml | kubectl apply -f - >/dev/null
kubectl -n axiom-bench apply -f - >/dev/null <<EOF
apiVersion: v1
kind: ServiceAccount
metadata:
  name: bench-client
  annotations:
    iam.gke.io/gcp-service-account: ${BENCHMARK_SERVICE_ACCOUNT}
---
apiVersion: batch/v1
kind: Job
metadata:
  name: managed-bench
spec:
  backoffLimit: 0
  activeDeadlineSeconds: 1800
  ttlSecondsAfterFinished: 300
  template:
    metadata:
      labels:
        app: axiom-managed-bench
    spec:
      restartPolicy: Never
      serviceAccountName: bench-client
      containers:
        - name: client
          image: ${CLIENT_IMAGE}
          imagePullPolicy: Always
          env:
            - {name: PROJECT_ID, value: "${PROJECT_ID}"}
            - {name: REGION, value: "${REGION}"}
            - {name: RUN_ID, value: "${RUN_ID}"}
            - {name: TAPE_URL, value: "http://tape.tape.svc.cluster.local:7137"}
            - {name: DEFER_URL, value: "http://defer.defer.svc.cluster.local:7141"}
            - {name: RELAY_URL, value: "http://relay.relay.svc.cluster.local:7000"}
            - {name: TAPE_READY_REPLICAS, value: "${TAPE_READY_REPLICAS}"}
            - {name: DEFER_READY_REPLICAS, value: "${DEFER_READY_REPLICAS}"}
            - {name: RELAY_READY_REPLICAS, value: "${RELAY_READY_REPLICAS}"}
            - {name: RECEIVER_URL, value: "${RECEIVER_URL}"}
            - {name: RECEIVER_SECRET, value: "${RECEIVER_SECRET}"}
            - {name: PUBSUB_TOPIC, value: "${PUBSUB_TOPIC}"}
            - {name: PUBSUB_SUBSCRIPTIONS, value: "${PUBSUB_SUBSCRIPTIONS}"}
            - {name: CLOUD_TASKS_QUEUE, value: "${CLOUD_TASKS_QUEUE}"}
            - {name: TAPE_EVENTS, value: "${TAPE_EVENTS:-5000}"}
            - {name: TAPE_PREPARE_CONCURRENCY, value: "${TAPE_PREPARE_CONCURRENCY:-64}"}
            - {name: TASKS_PER_SAMPLE, value: "${TASKS_PER_SAMPLE:-200}"}
            - {name: TASK_CREATE_CONCURRENCY, value: "${TASK_CREATE_CONCURRENCY:-64}"}
            - {name: RELAY_MESSAGES, value: "${RELAY_MESSAGES:-200}"}
            - {name: RELAY_BATCH_SIZE, value: "${RELAY_BATCH_SIZE:-25}"}
          resources:
            requests:
              cpu: 1
              memory: 1Gi
            limits:
              memory: 1Gi
EOF

deadline=$(( $(date +%s) + 1800 ))
while [[ $(date +%s) -lt "$deadline" ]]; do
  complete="$(kubectl -n axiom-bench get job managed-bench -o jsonpath='{.status.conditions[?(@.type=="Complete")].status}' 2>/dev/null || true)"
  failed="$(kubectl -n axiom-bench get job managed-bench -o jsonpath='{.status.conditions[?(@.type=="Failed")].status}' 2>/dev/null || true)"
  if [[ "$complete" == "True" ]]; then
    report="$(kubectl -n axiom-bench logs job/managed-bench)"
    printf '%s\n' "$report"
    if jq -e '.status == "passed"' >/dev/null <<<"$report"; then
      exit 0
    fi
    echo "benchmark completed with one or more failed comparisons" >&2
    exit 1
  fi
  if [[ "$failed" == "True" ]]; then
    kubectl -n axiom-bench get pod -l job-name=managed-bench -o wide >&2 || true
    kubectl -n axiom-bench logs job/managed-bench --all-containers=true || true
    exit 1
  fi
  sleep 5
done

kubectl -n axiom-bench get pod -l job-name=managed-bench -o wide || true
kubectl -n axiom-bench logs job/managed-bench --all-containers=true || true
echo "benchmark Job did not complete within 30 minutes" >&2
exit 1
