# CCLab Meteor Specifications

This directory contains specifications for **cclab-meteor**, a high-performance distributed task queue for Python.

## Core Features

- **[Workflow Orchestration](./workflows.md)**: Defines task composition primitives (Chain, Group, Chord).
- **[K8s Job Executor](./k8s-job-executor.md)**: Support for executing resource-intensive tasks in Kubernetes Jobs.
- **[Workflow State Machine](./workflow-state-machine.md)**: Lifecycle management for distributed and offloaded tasks.

## Components

- **[Backend Metadata](./backend-metadata.md)**: Generic metadata storage for tracking workflow state.
- **[Workflow Continuation](./workflow-continuation.md)**: Logic for advancing sequential workflows across executors.
