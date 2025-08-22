# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/timvw/reqwest-openai-tracing/releases/tag/v0.1.0) - 2025-08-22

### Added

- initial implementation of reqwest-openai-tracing library

### Fixed

- disable crates.io publishing until async-openai dependency is resolved
- remove invalid version_commit_message field from release-plz config
- remove hardcoded service.name from middleware

### Other

- remove GITHUB_SECRETS_SETUP.md documentation
- add GitHub workflows for automated testing and releases
- add OpenTelemetry GenAI semantic conventions to README
- improve README with langfuse integration details
- remove langfuse_auth example
- simplify langfuse authentication API
- add MIT and Apache 2.0 licenses
