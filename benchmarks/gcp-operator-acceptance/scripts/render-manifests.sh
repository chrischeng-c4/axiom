#!/usr/bin/env bash
set -euo pipefail

: "${ACCEPTANCE_APPS:?ACCEPTANCE_APPS is required}"
: "${BACKUP_BUCKET:?BACKUP_BUCKET is required}"
: "${BACKUP_GSA_EMAIL:?BACKUP_GSA_EMAIL is required}"
: "${RUN_ID:?RUN_ID is required}"
: "${MANIFEST_DIR:?MANIFEST_DIR is required}"

if [[ "$ACCEPTANCE_APPS" == "tape" ]]; then
  : "${TAPE_CLI:?TAPE_CLI is required}"
  : "${TAPE_IMAGE:?TAPE_IMAGE digest reference is required}"

  [[ -x "$TAPE_CLI" ]] || {
    echo "deployment CLI is not executable: $TAPE_CLI" >&2
    exit 1
  }

  mkdir -p \
    "$MANIFEST_DIR/tape/operator" \
    "$MANIFEST_DIR/tape/instance"

  "$TAPE_CLI" k8s crd render --out "$MANIFEST_DIR/tape/crd.yaml"
  "$TAPE_CLI" k8s operator render --namespace tape-system \
    --out "$MANIFEST_DIR/tape/operator/operator.yaml"
  "$TAPE_CLI" k8s instance render --profile dev --name tape --namespace tape \
    --image "$TAPE_IMAGE" --out "$MANIFEST_DIR/tape/instance/tape.yaml"

  cat > "$MANIFEST_DIR/tape/operator/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - operator.yaml
patches:
  - target:
      group: apps
      version: v1
      kind: Deployment
      name: tape-operator
    patch: |-
      - op: replace
        path: /spec/template/spec/containers/0/image
        value: ${TAPE_IMAGE}
EOF

  cat > "$MANIFEST_DIR/tape/instance/identity.yaml" <<EOF
apiVersion: v1
kind: Namespace
metadata:
  name: tape
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: tape-backup
  namespace: tape
  annotations:
    iam.gke.io/gcp-service-account: ${BACKUP_GSA_EMAIL}
EOF

  # Unlike Lumen/Sift, the Tape CRD has no CR-native `backup` field for the
  # operator to reconcile into a CronJob (apps/tape/src/operator/crd.rs).
  # Hand-roll the same disposable-run shape directly against the verified
  # `tape backup --url ... --dest ... --retention-secs ...` CLI verb.
  # ENTRYPOINT is already the `tape` binary (images/Dockerfile.tape), so
  # `args` alone selects the subcommand — no `command` override.
  cat > "$MANIFEST_DIR/tape/instance/backup-cronjob.yaml" <<EOF
apiVersion: batch/v1
kind: CronJob
metadata:
  name: tape-backup
  namespace: tape
spec:
  schedule: "*/5 * * * *"
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: tape-backup
          restartPolicy: Never
          containers:
            - name: backup
              image: ${TAPE_IMAGE}
              args:
                - backup
                - --url
                - http://tape.tape.svc.cluster.local:7137
                - --dest
                - gs://${BACKUP_BUCKET}/tape/${RUN_ID}
                - --retention-secs
                - "3600"
EOF

  cat > "$MANIFEST_DIR/tape/instance/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - identity.yaml
  - tape.yaml
  - backup-cronjob.yaml
patches:
  - target:
      group: tape.dev
      version: v1alpha1
      kind: Tape
      name: tape
    patch: |-
      - op: add
        path: /spec/imagePullPolicy
        value: IfNotPresent
      - op: replace
        path: /spec/resources/cpu
        value: 500m
      - op: replace
        path: /spec/resources/memory
        value: 1Gi
      - op: replace
        path: /spec/storage
        value: 1Gi
EOF

  kubectl kustomize "$MANIFEST_DIR/tape/operator" > "$MANIFEST_DIR/tape/operator.bundle.yaml"
  kubectl kustomize "$MANIFEST_DIR/tape/instance" > "$MANIFEST_DIR/tape/instance.bundle.yaml"
else
  : "${LUMEN_CLI:?LUMEN_CLI is required}"
  : "${SIFT_CLI:?SIFT_CLI is required}"
  : "${LUMEN_IMAGE:?LUMEN_IMAGE digest reference is required}"
  : "${SIFT_IMAGE:?SIFT_IMAGE digest reference is required}"
  : "${GKE_CLUSTER_NAME:?GKE_CLUSTER_NAME is required}"
  : "${GKE_ZONE:?GKE_ZONE is required}"
  : "${PROJECT_ID:?PROJECT_ID is required}"

  for cli in "$LUMEN_CLI" "$SIFT_CLI"; do
    [[ -x "$cli" ]] || {
      echo "deployment CLI is not executable: $cli" >&2
      exit 1
    }
  done

  mkdir -p \
    "$MANIFEST_DIR/lumen/operator" \
    "$MANIFEST_DIR/lumen/instance" \
    "$MANIFEST_DIR/sift/operator" \
    "$MANIFEST_DIR/sift/instance" \
    "$MANIFEST_DIR/sift/collector"

  "$LUMEN_CLI" k8s crd render --out "$MANIFEST_DIR/lumen/crd.yaml"
  "$LUMEN_CLI" k8s operator render --namespace lumen-system \
    --out "$MANIFEST_DIR/lumen/operator/operator.yaml"
  "$LUMEN_CLI" k8s instance render --profile dev --name lumen --namespace lumen \
    --image "$LUMEN_IMAGE" --out "$MANIFEST_DIR/lumen/instance/lumen.yaml"

  "$SIFT_CLI" k8s crd render --out "$MANIFEST_DIR/sift/crd.yaml"
  "$SIFT_CLI" k8s operator render --namespace sift-system \
    --out "$MANIFEST_DIR/sift/operator/operator.yaml"
  "$SIFT_CLI" k8s instance render --profile dev \
    --out "$MANIFEST_DIR/sift/instance/sift.yaml"
  "$SIFT_CLI" k8s collector render --namespace sift --image "$SIFT_IMAGE" \
    --out "$MANIFEST_DIR/sift/collector/collector.yaml"

  cat > "$MANIFEST_DIR/lumen/operator/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - operator.yaml
patches:
  - target:
      group: apps
      version: v1
      kind: Deployment
      name: lumen-operator
    patch: |-
      - op: replace
        path: /spec/template/spec/containers/0/image
        value: ${LUMEN_IMAGE}
EOF

  cat > "$MANIFEST_DIR/lumen/instance/identity.yaml" <<EOF
apiVersion: v1
kind: Namespace
metadata:
  name: lumen
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: lumen-backup
  namespace: lumen
  annotations:
    iam.gke.io/gcp-service-account: ${BACKUP_GSA_EMAIL}
EOF

  cat > "$MANIFEST_DIR/lumen/instance/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - identity.yaml
  - lumen.yaml
patches:
  - target:
      group: lumen.dev
      version: v1alpha1
      kind: Lumen
      name: lumen
    patch: |-
      - op: add
        path: /spec/imagePullPolicy
        value: IfNotPresent
      - op: replace
        path: /spec/serving/cpu
        value: 500m
      - op: replace
        path: /spec/serving/memory
        value: 1Gi
      - op: replace
        path: /spec/logFormat
        value: json
      - op: add
        path: /spec/serving/raftStorage
        value: 1Gi
      - op: add
        path: /spec/serving/backup
        value:
          schedule: "*/5 * * * *"
          destination: gs://${BACKUP_BUCKET}/lumen/${RUN_ID}
          retentionSecs: 3600
EOF

  cat > "$MANIFEST_DIR/sift/operator/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - operator.yaml
patches:
  - target:
      group: apps
      version: v1
      kind: Deployment
      name: sift-operator
    patch: |-
      - op: replace
        path: /spec/template/spec/containers/0/image
        value: ${SIFT_IMAGE}
EOF

  cat > "$MANIFEST_DIR/sift/instance/identity.yaml" <<EOF
apiVersion: v1
kind: Namespace
metadata:
  name: sift
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: sift-backup
  namespace: sift
  annotations:
    iam.gke.io/gcp-service-account: ${BACKUP_GSA_EMAIL}
EOF

  cat > "$MANIFEST_DIR/sift/instance/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - identity.yaml
  - sift.yaml
patches:
  - target:
      group: sift.axiom.dev
      version: v1alpha1
      kind: Sift
      name: sift
    patch: |-
      - op: replace
        path: /metadata/namespace
        value: sift
      - op: replace
        path: /spec/image
        value: ${SIFT_IMAGE}
      - op: replace
        path: /spec/dataSize
        value: 1Gi
      - op: add
        path: /spec/backup
        value:
          schedule: "*/5 * * * *"
          destination: gs://${BACKUP_BUCKET}/sift/${RUN_ID}
          retentionSecs: 3600
EOF

  kubectl kustomize "$MANIFEST_DIR/lumen/operator" > "$MANIFEST_DIR/lumen/operator.bundle.yaml"
  kubectl kustomize "$MANIFEST_DIR/lumen/instance" > "$MANIFEST_DIR/lumen/instance.bundle.yaml"
  kubectl kustomize "$MANIFEST_DIR/sift/operator" > "$MANIFEST_DIR/sift/operator.bundle.yaml"
  kubectl kustomize "$MANIFEST_DIR/sift/instance" > "$MANIFEST_DIR/sift/instance.bundle.yaml"

  cat > "$MANIFEST_DIR/sift/collector/config.yaml" <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: sift-collector
  namespace: sift
data:
  endpoint: http://sift.sift.svc.cluster.local:7380
  project: operator-acceptance
  environment: gke
  gcpProjectId: ${PROJECT_ID}
  clusterName: ${GKE_CLUSTER_NAME}
  location: ${GKE_ZONE}
---
apiVersion: v1
kind: Secret
metadata:
  name: sift-collector
  namespace: sift
type: Opaque
stringData:
  token: ""
EOF

  cat > "$MANIFEST_DIR/sift/collector/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - config.yaml
  - collector.yaml
EOF

  kubectl kustomize "$MANIFEST_DIR/sift/collector" > "$MANIFEST_DIR/sift/collector.bundle.yaml"
fi
