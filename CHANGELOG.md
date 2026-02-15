## [1.2.2](https://github.com/statik/audience/compare/v1.2.1...v1.2.2) (2026-02-15)


### Bug Fixes

* **ci:** split automerge steps and use squash merge ([#32](https://github.com/statik/audience/issues/32)) ([1253183](https://github.com/statik/audience/commit/125318326d6dfe054aafd73fd2fe633f8f0bbc74))

## [1.2.1](https://github.com/statik/audience/compare/v1.2.0...v1.2.1) (2026-02-15)


### Performance Improvements

* **ci:** optimize release build speed and pin actions to SHA ([#16](https://github.com/statik/audience/issues/16)) ([aebdf41](https://github.com/statik/audience/commit/aebdf412995a7f87f88a9537ad7b23a3544c1aae))

# [1.2.0](https://github.com/statik/audience/compare/v1.1.0...v1.2.0) (2026-02-15)


### Features

* add simulated PTZ camera for dev testing and demos ([#14](https://github.com/statik/audience/issues/14)) ([07b63c0](https://github.com/statik/audience/commit/07b63c043772fc4326a805eeb7ffbe2348b3d50b))

# [1.1.0](https://github.com/statik/audience/compare/v1.0.3...v1.1.0) (2026-02-15)


### Features

* **ci:** add Scoop and Homebrew publishing to release pipeline ([#15](https://github.com/statik/audience/issues/15)) ([e94e509](https://github.com/statik/audience/commit/e94e50965fde411d78f63f3d1d78975a511327b7))

## [1.0.3](https://github.com/statik/audience/compare/v1.0.2...v1.0.3) (2026-02-15)


### Bug Fixes

* **ci:** enable globstar for release asset upload globs ([#12](https://github.com/statik/audience/issues/12)) ([0aac58f](https://github.com/statik/audience/commit/0aac58f34922a7680489fff81578a44ace4c967a))

## [1.0.2](https://github.com/statik/audience/compare/v1.0.1...v1.0.2) (2026-02-15)


### Bug Fixes

* enable macOS camera access in Tauri WebView ([#11](https://github.com/statik/audience/issues/11)) ([341bad2](https://github.com/statik/audience/commit/341bad2da2ec6c651f428082013ce3e926cba5ee))

## [1.0.1](https://github.com/statik/audience/compare/v1.0.0...v1.0.1) (2026-02-15)


### Bug Fixes

* **ci:** skip prek install in release workflow ([#10](https://github.com/statik/audience/issues/10)) ([e97ec81](https://github.com/statik/audience/commit/e97ec81896b7cc9c42cbd879763bb27a8f1e1337))
* **ci:** use explicit file globs for release asset upload ([#9](https://github.com/statik/audience/issues/9)) ([1205702](https://github.com/statik/audience/commit/12057022ed44b76388e6437021652e625306b67c))

# 1.0.0 (2026-02-15)


### Bug Fixes

* address code review issues (security, stale closures, validation) ([0f0a8e4](https://github.com/statik/audience/commit/0f0a8e4360f783b227e720014e59d566d165c83c))
* address second code review (deadlocks, NaN guards, stale closures) ([cf90ac4](https://github.com/statik/audience/commit/cf90ac49a3885a90c28bde2dc84d99adb8472743))
* auto-format Rust code and add justfile + CLAUDE.md ([ff1709b](https://github.com/statik/audience/commit/ff1709ba88e70dae28bc4c6677fcf32b0e79f63a))
* **ci:** disable husky hooks during semantic-release ([#5](https://github.com/statik/audience/issues/5)) ([29c1905](https://github.com/statik/audience/commit/29c1905ad3fc273154576768c9915ef961639c51))
* resolve all clippy warnings and fix tauri config ([e878429](https://github.com/statik/audience/commit/e87842982904a3bf9d3dce2bf6b9df95c39b52a2))
* resolve clippy derivable_impls warnings and add session-start hook ([7025eed](https://github.com/statik/audience/commit/7025eed6b3d1df72e6144f078fb7d64b9a8596e3))
* resolve TypeScript lint and compilation errors ([6a1bc91](https://github.com/statik/audience/commit/6a1bc9100315b3cfa2565ab3912edf5f330ba406))
* wire PtzDispatcher to AppState and add MJPEG server shutdown ([923a3ed](https://github.com/statik/audience/commit/923a3ede86647b0342900980bd51a658bbdd841c))


### Features

* initialize Tauri 2 + React + TypeScript project structure ([2ac11ab](https://github.com/statik/audience/commit/2ac11abbcf14926a18942bfdcc8296537d53ca73))
