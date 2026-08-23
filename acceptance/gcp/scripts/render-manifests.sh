#!/usr/bin/env bash
set -euo pipefail

: "${ACCEPTANCE_APPS:=lumen sift}"
: "${RUN_ID:?RUN_ID is required}"
: "${MANIFEST_DIR:?MANIFEST_DIR is required}"
: "${GKE_CLUSTER_NAME:?GKE_CLUSTER_NAME is required}"
: "${GKE_ZONE:?GKE_ZONE is required}"
: "${PROJECT_ID:?PROJECT_ID is required}"

# Determine acceptance mode and require only the appropriate CLIs/images
case "$ACCEPTANCE_APPS" in
  "lumen sift"|"lumen auth")
    : "${LUMEN_CLI:?LUMEN_CLI is required for lumen-sift mode}"
    : "${LUMEN_IMAGE:?LUMEN_IMAGE digest reference is required for lumen-sift mode}"
    [[ -x "$LUMEN_CLI" ]] || {
      echo "deployment CLI is not executable: $LUMEN_CLI" >&2
      exit 1
    }
    if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then
    : "${SIFT_CLI:?SIFT_CLI is required for lumen-sift mode}"
    : "${SIFT_IMAGE:?SIFT_IMAGE digest reference is required for lumen-sift mode}"
    [[ -x "$SIFT_CLI" ]] || {
      echo "deployment CLI is not executable: $SIFT_CLI" >&2
      exit 1
    }
    fi
    mkdir -p "$MANIFEST_DIR/lumen/operator" "$MANIFEST_DIR/lumen/instance"
    if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then
      mkdir -p "$MANIFEST_DIR/sift/operator" "$MANIFEST_DIR/sift/instance" "$MANIFEST_DIR/sift/collector"
    fi
    ;;
  "tape")
    : "${TAPE_CLI:?TAPE_CLI is required for tape mode}"
    : "${TAPE_IMAGE:?TAPE_IMAGE digest reference is required for tape mode}"
    [[ -x "$TAPE_CLI" ]] || {
      echo "deployment CLI is not executable: $TAPE_CLI" >&2
      exit 1
    }
    mkdir -p \
      "$MANIFEST_DIR/tape/operator" \
      "$MANIFEST_DIR/tape/instance"
    ;;
  *)
    echo "ACCEPTANCE_APPS must be 'lumen sift', 'lumen auth', or 'tape'" >&2
    exit 1
    ;;
esac

# Render Lumen manifests for both Lumen modes.
if [[ "$ACCEPTANCE_APPS" == "lumen sift" || "$ACCEPTANCE_APPS" == "lumen auth" ]]; then
  "$LUMEN_CLI" k8s crd render --out "$MANIFEST_DIR/lumen/crd.yaml"
  "$LUMEN_CLI" k8s operator render --namespace lumen-system \
    --out "$MANIFEST_DIR/lumen/operator/operator.yaml"
  "$LUMEN_CLI" k8s instance render --profile dev --name lumen --namespace lumen \
    --image "$LUMEN_IMAGE" --out "$MANIFEST_DIR/lumen/instance/lumen.yaml"

fi

if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then
  "$SIFT_CLI" k8s crd render --out "$MANIFEST_DIR/sift/crd.yaml"
  "$SIFT_CLI" k8s operator render --namespace sift-system \
    --out "$MANIFEST_DIR/sift/operator/operator.yaml"
  "$SIFT_CLI" k8s instance render --profile dev \
    --out "$MANIFEST_DIR/sift/instance/sift.yaml"
  "$SIFT_CLI" k8s collector render --namespace sift --image "$SIFT_IMAGE" \
    --out "$MANIFEST_DIR/sift/collector/collector.yaml"
fi

# Render Tape manifests for tape mode
if [[ "$ACCEPTANCE_APPS" == "tape" ]]; then
  "$TAPE_CLI" k8s crd render --out "$MANIFEST_DIR/tape/crd.yaml"
  "$TAPE_CLI" k8s operator render --namespace tape-system \
    --out "$MANIFEST_DIR/tape/operator/operator.yaml"
  "$TAPE_CLI" k8s instance render --profile dev --name tape --namespace tape \
    --image "$TAPE_IMAGE" --out "$MANIFEST_DIR/tape/instance/tape.yaml"
fi

if [[ "$ACCEPTANCE_APPS" == "lumen sift" || "$ACCEPTANCE_APPS" == "lumen auth" ]]; then
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
fi

if [[ "$ACCEPTANCE_APPS" == "lumen sift" || "$ACCEPTANCE_APPS" == "lumen auth" ]]; then
  if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then
    : "${BACKUP_BUCKET:?BACKUP_BUCKET is required}"
    : "${BACKUP_GSA_EMAIL:?BACKUP_GSA_EMAIL is required}"
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
  else
cat > "$MANIFEST_DIR/lumen/instance/identity.yaml" <<EOF
apiVersion: v1
kind: Namespace
metadata:
  name: lumen
EOF
  fi
  instance_resources="identity.yaml"

cat > "$MANIFEST_DIR/lumen/instance/kustomization.yaml" <<EOF
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  ${instance_resources:+- $instance_resources}
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
$(if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then cat <<PATCH
      - op: add
        path: /spec/serving/backup
        value:
          schedule: "*/5 * * * *"
          destination: gs://${BACKUP_BUCKET}/lumen/${RUN_ID}
          retentionSecs: 3600
PATCH
fi)
EOF

kubectl kustomize "$MANIFEST_DIR/lumen/operator" > "$MANIFEST_DIR/lumen/operator.bundle.yaml"
kubectl kustomize "$MANIFEST_DIR/lumen/instance" > "$MANIFEST_DIR/lumen/instance.bundle.yaml"

fi

if [[ "$ACCEPTANCE_APPS" == "lumen sift" ]]; then
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

# Tape-specific kustomization for tape mode
if [[ "$ACCEPTANCE_APPS" == "tape" ]]; then
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

# The Tape CRD DOES now carry a CR-native `backup` field (#2574), matching
# Lumen/Sift — the earlier note here ("run 0724154839: a /spec/backup patch is
# rejected by strict decoding") described the pre-#2574 CRD and is no longer
# true. The `tape-backup` ServiceAccount below is now also rendered by the
# operator; it stays here because this copy carries the Workload Identity
# annotation, which survives reconcile (a different field manager owns it).
#
# The CronJob itself is still hand-rolled, for one reason: the shared
# `service_k8s::render::cron_job` helper has no `suspend` knob, so a
# CR-native CronJob would be UNSUSPENDED. An unsuspended `*/5` schedule kept
# firing against the torn-down 1x1 instance during the cold-restore rebuild
# (run 0723080156) — see the `suspend: true` note below. Switching this to
# `/spec/backup` needs that knob first and a GKE run to re-prove the
# cold-restore phase; it is deliberately not a drive-by edit.
#
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
  # The acceptance flow triggers exactly one Job via
  # \`kubectl create job --from=cronjob/tape-backup\` (which works on a
  # suspended CronJob). An unsuspended schedule kept firing against the
  # torn-down 1x1 instance during the cold-restore rebuild (run 0723080156).
  suspend: true
  concurrencyPolicy: Forbid
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
      - op: add
        path: /spec/topics
        value:
          - name: acceptance
            subscriptions:
              - acceptance-sub
EOF

kubectl kustomize "$MANIFEST_DIR/tape/operator" > "$MANIFEST_DIR/tape/operator.bundle.yaml"
kubectl kustomize "$MANIFEST_DIR/tape/instance" > "$MANIFEST_DIR/tape/instance.bundle.yaml"
fi
