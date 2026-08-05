# Changelog

## [0.2.0](https://github.com/dallay/file-organizer/compare/v0.1.0...v0.2.0) (2026-08-05)


### Features

* add AgentSync instruction sync for root, Claude, and Copilot ([e31eb6d](https://github.com/dallay/file-organizer/commit/e31eb6d3606a4e40e66058abc12dc8c1843207c2))
* add organiza distribution pipeline (npm, release-please, crates.io, docker) ([5939f23](https://github.com/dallay/file-organizer/commit/5939f23155308bec24c5fb836e299f535c500b95))
* configurable categories, downloads auto-detect, and first-run fallback ([aaeb90d](https://github.com/dallay/file-organizer/commit/aaeb90ddd4518a8033bf30656af8a8a06f3bd1fa))
* configurable categories, downloads auto-detect, and first-run fallback ([1fadf2f](https://github.com/dallay/file-organizer/commit/1fadf2f94a2c70993fbba23a1cc31fa3e5efaa61))
* rebrand CLI to organiza and prepare release hygiene ([ce7cddb](https://github.com/dallay/file-organizer/commit/ce7cddb3ce27e29013d66201138fececafc16504))
* rebrand crate and binary to organiza ([8d48905](https://github.com/dallay/file-organizer/commit/8d48905d7b9a5a8579282ccd4e07ef4d7565d041))


### Bug Fixes

* address verify warnings W1-W4 ([3f99066](https://github.com/dallay/file-organizer/commit/3f99066c4aed60642e24dfd800a1113e71f84091))
* address verify warnings W1-W4 from configurable-categories cycle ([6d495f4](https://github.com/dallay/file-organizer/commit/6d495f441a2c5a82789f31c71dad9b3e91b4d384))
* apply CodeRabbit auto-fixes ([e3057fd](https://github.com/dallay/file-organizer/commit/e3057fd356a48cf698ae850e131fe56cba7ab53f))
* **ci:** pass required toolchain input and harden agentsync invocation ([827dd91](https://github.com/dallay/file-organizer/commit/827dd91c6b7cee64436b78a52165775db015e0e9))
* **ci:** resolve cross-platform CI failures ([e25e415](https://github.com/dallay/file-organizer/commit/e25e41526c98f6712b3b7e4a104f4ff2a284be83))
* make config path regression test separator-agnostic ([951b88a](https://github.com/dallay/file-organizer/commit/951b88ae41063602b3382986b988e8042e9bace3))
* pin darwin-arm64 optional dependency to exact version ([997bb6e](https://github.com/dallay/file-organizer/commit/997bb6e8912edf5a421b7c4a5f0d4ba05fa31da0))


### Documentation

* add distribution-pipeline change artifacts ([a9e9349](https://github.com/dallay/file-organizer/commit/a9e9349518f75b7f66c51e53449a7cd086e491f2))
* archive distribution-pipeline change and sync specs ([d1957c9](https://github.com/dallay/file-organizer/commit/d1957c9210fc609c220cc7271577e065a76bdf09))
* clarify setup instructions require a source checkout ([8c560bd](https://github.com/dallay/file-organizer/commit/8c560bdcec298df33af54dd4d227a0ff4019556d))
* document per-OS tooling installation ([ad0a15d](https://github.com/dallay/file-organizer/commit/ad0a15dc53f48b98686fdd13253d559dfe87d866))
* mark apply phase complete in state.yaml ([75f56e8](https://github.com/dallay/file-organizer/commit/75f56e8fcb81692b9f7efa9cf00e5dfc5df0ef58))
* mark PR2 tasks complete in distribution-pipeline change ([50d6ed8](https://github.com/dallay/file-organizer/commit/50d6ed8588060d80d66bf395b3853b0b353463ee))
* record verify PASS WITH WARNINGS after F1 fix ([a3b149e](https://github.com/dallay/file-organizer/commit/a3b149ef1bc3fa0f2a77b7e7dcd3d270c1e0768f))


### Build System

* add multi-stage docker image for organiza ([2ad8af2](https://github.com/dallay/file-organizer/commit/2ad8af2b7b8b7bff9e2469f7843152d90c67d1ba))
* add organiza npm wrapper and version-sync scripts ([eb673f2](https://github.com/dallay/file-organizer/commit/eb673f2e15eed479979774ad82367f2e3b1ca46c))


### Continuous Integration

* add organiza release pipeline with npm, crates.io, and Docker publishing ([941c4bf](https://github.com/dallay/file-organizer/commit/941c4bf079f42728611dca00567ef4ae3fde61d6))
* add quality, test matrix, and AgentSync drift jobs ([2bc2d15](https://github.com/dallay/file-organizer/commit/2bc2d156fdccfeb49258bd471061576f64a5eb1e))
* fix release-please github app token secret names ([6562d82](https://github.com/dallay/file-organizer/commit/6562d82995d642e36277a14fbac62ee97524b2e8))
* fix release-please github app token secret names ([0a0ceb4](https://github.com/dallay/file-organizer/commit/0a0ceb4c0da2c83e4ec922834539d9f206557f77))


### Tests

* drop version assertion in package_is_named_organiza ([4d690b1](https://github.com/dallay/file-organizer/commit/4d690b1e71c3a73d39e8fe0203822ab825f10eeb))
* isolate env in default_config_path_uses_organiza_directory ([fa859bd](https://github.com/dallay/file-organizer/commit/fa859bd0ee7091349d9f66562680dfab00dcd4e9))


### Chores

* add Lefthook hooks and Rust toolchain pin ([9ea7e5c](https://github.com/dallay/file-organizer/commit/9ea7e5c1340ea9b3d9deb37fa6f23b6078ae6742))
* add renovate configuration for dependency updates ([73713e4](https://github.com/dallay/file-organizer/commit/73713e464f55805bc82cf9e2ee188c8457402ccf))
* add renovate configuration for dependency updates ([27490a4](https://github.com/dallay/file-organizer/commit/27490a41f6cf9a8d0956d6d0a774816ed80dc8b1))
* reconcile gitignore managed block ([715a4cc](https://github.com/dallay/file-organizer/commit/715a4ccd17b0607f2080838ed76e1590abad5b15))
* rename launchers, config paths, and docs to organiza ([6faf2a9](https://github.com/dallay/file-organizer/commit/6faf2a9542eb5b2ed30e2c5bc0a8d7cb1412f9f2))
