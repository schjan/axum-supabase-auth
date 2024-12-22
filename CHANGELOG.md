# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/schjan/axum-supabase-auth/compare/v0.1.0...v0.2.0) - 2024-12-22

### Added

- move example to subcrate and cleanup dependencies (#19)
- API client (#11)

### Fixed

- *(deps)* update rust crate bon to v3 (#46)
- *(deps)* update rust crate thiserror to v2 (#42)
- *(deps)* update rust crate reqwest to v0.12.8 (#13)

### Other

- *(deps)* update codecov/codecov-action action to v5 (#48)
- *(deps)* update rust crate fake to v3 (#32)
- update dependencies
- update dependencies
- *(deps)* update rust crate clap to v4.5.19 (#14)
- update dependencies
- *(deps)* update rust crate axum to v0.7.7 (#12)
- add lockfile
- *(deps)* update rustsec/audit-check action to v2 (#10)
- clippy results in security tab (#9)

## [0.1.0](https://github.com/schjan/axum-supabase-auth/releases/tag/v0.1.0) - 2024-09-21

### Added

- implement and test supabase auth api client ([#7](https://github.com/schjan/axum-supabase-auth/pull/7))
- add ApiErrorResponse type
- refactor API client
- impl AsRef<User> for SignUpResponse
- impl API client SignUp
- add CI ([#1](https://github.com/schjan/axum-supabase-auth/pull/1))
- initial PoC import

### Other

- Add renovate.json ([#4](https://github.com/schjan/axum-supabase-auth/pull/4))
- add license
- refactor sign_in_up_body
- remove unused file
- refactor
- also run integration test for coverage
- add gotrue docker-compose from https://github.com/supabase-community/gotrue-go/tree/main/integration_test/setup
- try not to publish crate yet
- fix release-plz
