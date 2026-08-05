# Container Distribution Specification

## Purpose

Ship `organiza` as a multi-arch container image on Docker Hub (`yacosta738/organiza`) and GHCR (`ghcr.io/dallay/organiza`), built from a multi-stage musl Dockerfile, running as a non-root user under tini.

## Requirements

### Requirement: Image naming and tags

The pipeline MUST publish images to `yacosta738/organiza` (Docker Hub) and `ghcr.io/dallay/organiza` (GHCR). Each release MUST tag images with the exact semver and `latest`.

#### Scenario: Release tags

- GIVEN a release at vX.Y.Z
- WHEN images are pushed
- THEN both registries carry `vX.Y.Z` and `latest` tags

### Requirement: Multi-arch build

Images MUST support linux/amd64 and linux/arm64, built with Docker buildx/QEMU from a single manifest.

#### Scenario: Cross-arch pull

- GIVEN the image is published
- WHEN a user pulls on amd64 and on arm64
- THEN each host runs the native architecture image

### Requirement: Runtime safety

The image MUST run as a non-root user and MUST use tini as PID 1. The binary MUST be statically linked musl.

#### Scenario: Non-root and tini

- GIVEN the container starts
- WHEN the process list is inspected
- THEN PID 1 is tini and the user is non-root

### Requirement: Build hygiene and credentials

The Dockerfile MUST be a multi-stage build with a minimal final stage. `.dockerignore` MUST exclude build artifacts (target/, node_modules/, git metadata). Docker Hub push MUST use `DOCKERHUB_USERNAME`/`DOCKERHUB_TOKEN`; GHCR push MUST use the org PAT or fall back to repo-scoped `GITHUB_TOKEN`.

#### Scenario: Registry auth failure

- GIVEN Docker Hub or GHCR credentials are invalid
- WHEN the pipeline pushes
- THEN the push fails and the pipeline halts without a partial publish
