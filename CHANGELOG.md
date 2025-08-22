# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/timvw/reqwest-openai-tracing/releases/tag/v0.1.0) - 2025-08-22

Initial release of reqwest-openai-tracing - OpenTelemetry tracing middleware for OpenAI API calls made with reqwest.

### Features

- Automatic OpenTelemetry span creation for OpenAI API calls
- Support for chat completions, embeddings, completions, images, and audio operations
- Token usage tracking in span attributes
- Langfuse integration via OpenTelemetry with helper functions
- Context attributes support (session_id, user_id, tags, metadata)
- Compatible with async-openai library via HttpClient trait
- Works with any OpenTelemetry backend (Jaeger, Zipkin, Langfuse, etc.)
- Follows OpenTelemetry GenAI semantic conventions

### Documentation

- Comprehensive README with examples
- Helper functions for Langfuse configuration
- Example code for basic usage, Langfuse integration, and context attributes

### Infrastructure

- GitHub Actions workflows for automated testing and releases
- Release automation with release-plz
- MIT and Apache 2.0 dual licensing
