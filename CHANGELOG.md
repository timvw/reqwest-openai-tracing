# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/timvw/reqwest-openai-tracing/releases/tag/v0.1.0) - 2025-08-22

### Added

- OpenTelemetry tracing middleware for OpenAI API calls
- Support for chat completions, embeddings, image generation, and audio operations
- Automatic span creation following OpenTelemetry GenAI semantic conventions
- Token usage tracking in span attributes
- Langfuse integration via OpenTelemetry OTLP endpoint
- Context attributes support (session_id, user_id, tags, metadata)
- Helper functions for Langfuse authentication and configuration
- Compatibility with async-openai library via HttpClient trait
- Support for both OpenAI and Azure OpenAI endpoints
- Comprehensive examples for basic usage, Langfuse integration, and context attributes
- GitHub Actions workflows for testing and releases
- MIT and Apache 2.0 dual licensing
