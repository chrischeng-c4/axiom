#!/usr/bin/env ruby

require "base64"
require "digest"
require "json"
require "optparse"
require "yaml"

EXPECTED_IMAGE = "docker.io/curlimages/curl@sha256:7c12af72ceb38b7432ab85e1a265cff6ae58e06f95539d539b654f2cfa64bb13"
MANAGED_BY = "lumen-standalone-gke-acceptance"
PROGRAM_SHA256 = {
  "tooling" => "6972662b5a51c2fa4e0b8892fdb1f75cd0579ff3e9682c85a4984f07f95516a5",
  "api" => "40c430ae8b60dd7c441c16ac62da0a4d3f10de1f5028e94ddabb77c5219fdc40",
  "metrics" => "f9ded66f903d5baaf299fe28af1b9f543b37d9ddc3c3028161ce2933aa86caef",
}.freeze

def fail_contract(message)
  warn("kustomize contract: #{message}")
  exit(1)
end

def load_documents(path)
  raw = File.binread(path)
  stripped = raw.lstrip
  if stripped.start_with?("[")
    value = JSON.parse(raw)
    return value if value.is_a?(Array)
    return [value]
  end

  parts = raw.split(/^---[ \t]*\r?\n/)
  parts.each_with_object([]) do |part, documents|
    next if part.strip.empty?
    documents << YAML.safe_load(part, permitted_classes: [], permitted_symbols: [], aliases: false)
  end
rescue JSON::ParserError, Psych::Exception => e
  fail_contract("cannot parse #{path}: #{e.message}")
end

def expect(condition, message)
  fail_contract(message) unless condition
end

def expect_keys(value, keys, message)
  expect(value.is_a?(Hash) && value.keys.sort == keys.sort, message)
end

def labels_for(run_id, job = nil)
  labels = {
    "app.kubernetes.io/managed-by" => MANAGED_BY,
    "lumen.axiom.dev/gke-acceptance-run" => run_id,
  }
  labels["lumen.axiom.dev/gke-acceptance-job"] = job if job
  labels
end

def assert_no_forbidden_fields(pod, container)
  %w[hostNetwork hostPID hostIPC initContainers ephemeralContainers].each do |field|
    expect(!pod.key?(field), "forbidden pod field #{field}")
  end
  %w[envFrom resources].each do |field|
    expect(!container.key?(field), "forbidden container field #{field}")
  end
end

def assert_common_job(document, options, account:, automount:, container_name:, projected: false)
  expect_keys(document, %w[apiVersion kind metadata spec], "job fields changed")
  expect(document["apiVersion"] == "batch/v1", "job apiVersion changed")
  expect(document["kind"] == "Job", "expected one Job")
  metadata = document.fetch("metadata")
  expect_keys(metadata, %w[labels name namespace], "job metadata fields changed")
  expect(metadata["name"] == options.fetch(:job), "job name changed")
  expect(metadata["namespace"] == options.fetch(:client_namespace), "job namespace changed")
  expect(metadata["labels"] == labels_for(options.fetch(:run_id), options.fetch(:job)), "job labels changed")

  spec = document.fetch("spec")
  expect_keys(spec, %w[activeDeadlineSeconds backoffLimit template], "job spec fields changed")
  expect(spec["backoffLimit"] == 0, "backoffLimit changed")
  expect(spec["activeDeadlineSeconds"] == 120, "activeDeadlineSeconds changed")
  template = spec.fetch("template")
  expect_keys(template, %w[metadata spec], "Pod template fields changed")
  expect_keys(template.fetch("metadata"), %w[labels], "Pod metadata fields changed")
  expect(template.dig("metadata", "labels") == labels_for(options.fetch(:run_id), options.fetch(:job)), "Pod labels changed")
  pod = template.fetch("spec")
  expect_keys(
    pod,
    %w[automountServiceAccountToken containers restartPolicy securityContext serviceAccountName volumes],
    "Pod spec fields changed",
  )
  expect(pod["serviceAccountName"] == account, "service account changed")
  expect(pod["automountServiceAccountToken"] == automount, "automount token changed")
  expect(pod["restartPolicy"] == "Never", "restart policy changed")
  expect(pod["securityContext"] == {
    "runAsNonRoot" => true,
    "runAsUser" => 100,
    "runAsGroup" => 101,
    "seccompProfile" => {"type" => "RuntimeDefault"},
  }, "Pod security context changed")
  expect(pod.fetch("containers").length == 1, "job must have one container")
  container = pod.fetch("containers").first
  expected_container_keys = %w[args command image name securityContext volumeMounts]
  expected_container_keys << "env" unless container_name == "tooling"
  expect_keys(container, expected_container_keys, "container fields changed")
  expect(container["name"] == container_name, "container name changed")
  expect(container["image"] == EXPECTED_IMAGE, "client image changed")
  expect(container["command"] == ["/bin/sh", "-ec"], "container command changed")
  expect(container["args"].is_a?(Array) && container["args"].length == 1, "container args changed")
  expect(
    Digest::SHA256.hexdigest(container["args"].first) == PROGRAM_SHA256.fetch(container_name),
    "container program changed",
  )
  expect(container["securityContext"] == {
    "allowPrivilegeEscalation" => false,
    "readOnlyRootFilesystem" => true,
    "capabilities" => {"drop" => ["ALL"]},
  }, "container security context changed")
  assert_no_forbidden_fields(pod, container)

  expected_volumes = [{"name" => "memory", "emptyDir" => {"medium" => "Memory"}}]
  expected_mounts = [{"name" => "memory", "mountPath" => "/run/lumen"}]
  if projected
    expected_volumes << {
      "name" => "projected",
      "projected" => {
        "sources" => [{
          "serviceAccountToken" => {
            "path" => "token",
            "audience" => "lumen.axiom.dev",
            "expirationSeconds" => 600,
          },
        }],
      },
    }
    expected_mounts << {
      "name" => "projected",
      "mountPath" => "/run/lumen/projected",
      "readOnly" => true,
    }
  end
  expect(pod["volumes"] == expected_volumes, "volume contract changed")
  expect(container["volumeMounts"] == expected_mounts, "volume mount contract changed")
  container
end

def require_options(options, keys)
  keys.each do |key|
    fail_contract("missing --#{key.to_s.tr("_", "-")}") if options[key].nil?
  end
end

def assert_client_documents(documents, options)
  expect(documents.length == 3, "client render must have three resources")
  namespace = documents.find { |doc| doc["kind"] == "Namespace" }
  accounts = documents.select { |doc| doc["kind"] == "ServiceAccount" }
  expect(!namespace.nil? && accounts.length == 2, "client resource inventory changed")
  expect_keys(namespace, %w[apiVersion kind metadata], "Namespace fields changed")
  expect(namespace["apiVersion"] == "v1" && namespace["kind"] == "Namespace", "Namespace identity changed")
  expect_keys(namespace.fetch("metadata"), %w[labels name], "Namespace metadata fields changed")
  expect(namespace.dig("metadata", "name") == options.fetch(:client_namespace), "Namespace name changed")
  expected_namespace_labels = labels_for(options.fetch(:run_id)).merge(
    "pod-security.kubernetes.io/enforce" => "restricted",
    "pod-security.kubernetes.io/audit" => "restricted",
    "pod-security.kubernetes.io/warn" => "restricted",
  )
  expect(namespace.dig("metadata", "labels") == expected_namespace_labels, "Namespace labels changed")
  expect(accounts.map { |doc| doc.dig("metadata", "name") }.sort == %w[app unlisted], "ServiceAccount names changed")
  accounts.each do |account|
    expect_keys(account, %w[apiVersion kind metadata], "ServiceAccount fields changed")
    expect(account["apiVersion"] == "v1" && account["kind"] == "ServiceAccount", "ServiceAccount identity changed")
    expect_keys(account.fetch("metadata"), %w[labels name namespace], "ServiceAccount metadata fields changed")
    expect(account.dig("metadata", "namespace") == options.fetch(:client_namespace), "ServiceAccount namespace changed")
    expect(account.dig("metadata", "labels") == labels_for(options.fetch(:run_id)), "ServiceAccount labels changed")
  end
end

def assert_tooling_document(document, options)
  container = assert_common_job(document, options, account: "default", automount: false, container_name: "tooling")
  expect(!container.key?("env"), "tooling env changed")
  program = container.fetch("args").fetch(0)
  ["command -v curl", "command -v base64", "command -v grep", "row=client-tools"].each do |needle|
    expect(program.include?(needle), "tooling command changed")
  end
end

def token_row(options)
  matrix = {
    ["default", "app"] => [true, false],
    ["default", "unlisted"] => [true, false],
    ["projected", "app"] => [false, true],
    ["projected", "unlisted"] => [false, true],
    ["missing", "default"] => [false, false],
    ["bad", "unlisted"] => [false, false],
  }
  expected = matrix[[options.fetch(:token_mode), options.fetch(:account)]]
  fail_contract("invalid token/account row") if expected.nil?
  expected
end

def assert_api_document(document, options)
  require_options(options, %i[job account token_mode runtime_namespace service method path request_file expected_status required_id rejected_id row_label])
  expected = token_row(options)
  container = assert_common_job(
    document,
    options,
    account: options.fetch(:account),
    automount: expected.fetch(0),
    container_name: "api",
    projected: expected.fetch(1),
  )
  required_id = options.fetch(:required_id) == "none" ? "" : options.fetch(:required_id)
  rejected_id = options.fetch(:rejected_id) == "none" ? "" : options.fetch(:rejected_id)
  request_body = Base64.strict_encode64(File.binread(options.fetch(:request_file)))
  expected_env = [
    ["LUMEN_TOKEN_MODE", options.fetch(:token_mode)],
    ["LUMEN_RUNTIME_NAMESPACE", options.fetch(:runtime_namespace)],
    ["LUMEN_SERVICE", options.fetch(:service)],
    ["LUMEN_METHOD", options.fetch(:method)],
    ["LUMEN_PATH", options.fetch(:path)],
    ["LUMEN_REQUEST_BODY_B64", request_body],
    ["LUMEN_EXPECTED_STATUS", options.fetch(:expected_status)],
    ["LUMEN_REQUIRED_ID", required_id],
    ["LUMEN_REJECTED_ID", rejected_id],
    ["LUMEN_ROW_LABEL", options.fetch(:row_label)],
    ["LUMEN_BAD_TOKEN", "invalid-gke-token"],
  ].map { |name, value| {"name" => name, "value" => value} }
  expect(container["env"] == expected_env, "API env contract changed")
  program = container.fetch("args").fetch(0)
  [
    "base64 -d >\"$work/request.json\"",
    "--header @\"$work/header\"",
    "--header 'content-type: application/json'",
    "--data-binary @\"$work/request.json\"",
    "http://${LUMEN_SERVICE}.${LUMEN_RUNTIME_NAMESPACE}.svc.cluster.local:7373${LUMEN_PATH}",
    "/var/run/secrets/kubernetes.io/serviceaccount/token",
    "/run/lumen/projected/token",
    "LUMEN_BAD_TOKEN",
  ].each { |needle| expect(program.include?(needle), "API command changed") }
end

def assert_metrics_document(document, options)
  require_options(options, %i[job runtime_namespace service row_label])
  container = assert_common_job(document, options, account: "default", automount: false, container_name: "metrics")
  expected_env = [
    {"name" => "LUMEN_RUNTIME_NAMESPACE", "value" => options.fetch(:runtime_namespace)},
    {"name" => "LUMEN_SERVICE", "value" => options.fetch(:service)},
    {"name" => "LUMEN_ROW_LABEL", "value" => options.fetch(:row_label)},
  ]
  expect(container["env"] == expected_env, "metrics env contract changed")
  program = container.fetch("args").fetch(0)
  ["--write-out '%{http_code}'", "test \"$status\" = 200", "/metrics", "cat \"$work/metrics\""].each do |needle|
    expect(program.include?(needle), "metrics command changed")
  end
end

def assert_bundle(documents, options)
  require_options(options, %i[tooling_job api_job metrics_job metrics_row_label account token_mode runtime_namespace service method path request_file expected_status required_id rejected_id row_label])
  expect(options[:tooling_job] != options[:api_job] && options[:api_job] != options[:metrics_job] && options[:tooling_job] != options[:metrics_job], "bundle job names must be unique")
  expect(options[:metrics_row_label] != options[:row_label], "bundle row labels must be distinct")
  expect(documents.length == 6, "bundle must have six resources")
  client_documents = documents.select { |doc| %w[Namespace ServiceAccount].include?(doc["kind"]) }
  job_documents = documents.select { |doc| doc["kind"] == "Job" }
  expect(client_documents.length == 3 && job_documents.length == 3, "bundle resource inventory changed")
  assert_client_documents(client_documents, options)
  expected_job_names = [options.fetch(:tooling_job), options.fetch(:api_job), options.fetch(:metrics_job)].sort
  expect(job_documents.map { |doc| doc.dig("metadata", "name") }.sort == expected_job_names, "bundle job names changed")
  tooling_doc = job_documents.find { |doc| doc.dig("metadata", "name") == options.fetch(:tooling_job) }
  api_doc = job_documents.find { |doc| doc.dig("metadata", "name") == options.fetch(:api_job) }
  metrics_doc = job_documents.find { |doc| doc.dig("metadata", "name") == options.fetch(:metrics_job) }
  assert_tooling_document(tooling_doc, options.merge(job: options.fetch(:tooling_job)))
  assert_api_document(api_doc, options.merge(job: options.fetch(:api_job)))
  assert_metrics_document(metrics_doc, options.merge(job: options.fetch(:metrics_job), row_label: options.fetch(:metrics_row_label)))
end

options = {emit_json: false}
component = ARGV.shift
parser = OptionParser.new do |opts|
  opts.on("--file PATH") { |value| options[:file] = value }
  opts.on("--client-namespace VALUE") { |value| options[:client_namespace] = value }
  opts.on("--runtime-namespace VALUE") { |value| options[:runtime_namespace] = value }
  opts.on("--service VALUE") { |value| options[:service] = value }
  opts.on("--run-id VALUE") { |value| options[:run_id] = value }
  opts.on("--job VALUE") { |value| options[:job] = value }
  opts.on("--account VALUE") { |value| options[:account] = value }
  opts.on("--token-mode VALUE") { |value| options[:token_mode] = value }
  opts.on("--method VALUE") { |value| options[:method] = value }
  opts.on("--path VALUE") { |value| options[:path] = value }
  opts.on("--request-file PATH") { |value| options[:request_file] = value }
  opts.on("--expected-status VALUE") { |value| options[:expected_status] = value }
  opts.on("--required-id VALUE") { |value| options[:required_id] = value }
  opts.on("--rejected-id VALUE") { |value| options[:rejected_id] = value }
  opts.on("--row-label VALUE") { |value| options[:row_label] = value }
  opts.on("--tooling-job VALUE") { |value| options[:tooling_job] = value }
  opts.on("--api-job VALUE") { |value| options[:api_job] = value }
  opts.on("--metrics-job VALUE") { |value| options[:metrics_job] = value }
  opts.on("--metrics-row-label VALUE") { |value| options[:metrics_row_label] = value }
  opts.on("--emit-json") { options[:emit_json] = true }
end
parser.parse!(ARGV)
fail_contract("unexpected positional arguments") unless ARGV.empty?
fail_contract("unknown component") unless %w[client tooling api metrics bundle].include?(component)
%i[file client_namespace run_id].each { |key| fail_contract("missing --#{key.to_s.tr("_", "-")}") if options[key].nil? }

documents = load_documents(options.fetch(:file))
expect(!documents.empty?, "empty Kubernetes render")
expect(!JSON.generate(documents).include?("INVALID_"), "unresolved sentinel")
allowed_pairs = [["v1", "Namespace"], ["v1", "ServiceAccount"], ["batch/v1", "Job"]]
documents.each do |document|
  expect(document.is_a?(Hash), "resource is not an object")
  expect(allowed_pairs.include?([document["apiVersion"], document["kind"]]), "forbidden resource kind")
end

case component
when "client"
  assert_client_documents(documents, options)
when "tooling"
  expect(documents.length == 1, "tooling render must have one resource")
  require_options(options, %i[job])
  assert_tooling_document(documents.first, options)
when "api"
  expect(documents.length == 1, "API render must have one resource")
  assert_api_document(documents.first, options)
when "metrics"
  expect(documents.length == 1, "metrics render must have one resource")
  assert_metrics_document(documents.first, options)
when "bundle"
  assert_bundle(documents, options)
end

puts(JSON.generate(documents)) if options[:emit_json]
