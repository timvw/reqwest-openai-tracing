# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.0](https://github.com/timvw/reqwest-openai-tracing/releases/tag/v0.0.0) - 2025-08-22

### Added

- enforce linear history and document squash merge settings
- add version-controlled branch protection configurations
- initial implementation of reqwest-openai-tracing library

### Fixed

- remove invalid registry field from release-plz config
- configure release-plz to work without crates.io registry
- disable crates.io publishing until async-openai dependency is resolved
- remove invalid version_commit_message field from release-plz config
- remove hardcoded service.name from middleware

### Other

- reset version to 0.0.0 after removing premature release
- remove manual configuration files in favor of CLI setup
- simplify branch protection to use only GitHub Rulesets
- remove GITHUB_SECRETS_SETUP.md documentation
- add GitHub workflows for automated testing and releases
- add OpenTelemetry GenAI semantic conventions to README
- improve README with langfuse integration details
- remove langfuse_auth example
- simplify langfuse authentication API
- add MIT and Apache 2.0 licenses
