# Changelog

## [0.4.2](https://github.com/us/null-e/compare/v0.4.1...v0.4.2) (2026-06-18)


### Features

* **core:** faster, safer scanning and trash reclamation ([1c273c7](https://github.com/us/null-e/commit/1c273c7a07ea51ae84e1623e8e7255a2c67027cc))
* **ui:** Pastel Bento redesign and sidebar results layout ([8493346](https://github.com/us/null-e/commit/8493346f7d888ceac34f7264ecc13bc9b39132b6))

## [0.4.1](https://github.com/us/null-e/compare/v0.4.0...v0.4.1) (2026-03-30)


### Bug Fixes

* **ci:** add explicit tag_name to softprops/action-gh-release ([7f7230e](https://github.com/us/null-e/commit/7f7230ee4dd308f825fcd66dacd38d25c57259b1))
* **ci:** add tag input to release workflow_dispatch ([716e4a3](https://github.com/us/null-e/commit/716e4a34f560ebf64500db51c8c8eebb3486b577))
* **ci:** exclude Tauri GUI from CLI release build ([9492ce9](https://github.com/us/null-e/commit/9492ce90ce0f35ca96f8ffd6c30c3f5a46dfbce7))
* **ci:** pre-build frontend and skip beforeBuildCommand in tauri-action ([fff0a0f](https://github.com/us/null-e/commit/fff0a0fc904eafed883556280f525f20edf76f5b))
* **ci:** trigger release workflow via workflow_dispatch from release-please ([864b61d](https://github.com/us/null-e/commit/864b61d1f0ff73d444a1a255aa041c12c7bd1aa5))
* **ci:** use JSON format for tauri --config override ([44ded58](https://github.com/us/null-e/commit/44ded5885624ec26ba1eeb6c39679f827b522367))
* **ci:** use RELEASE_TAG env for workflow_dispatch compatibility ([0a792ad](https://github.com/us/null-e/commit/0a792ad0fadf037a62b35554a20d1476a387ecbf))

## [0.4.0](https://github.com/us/null-e/compare/v0.3.0...v0.4.0) (2026-03-28)


### ⚠ BREAKING CHANGES

* project restructured as Cargo workspace with Tauri GUI crate

### Features

* add Tauri GUI with full UX, auto-update, system tray, and cross-platform releases ([1e0b5f9](https://github.com/us/null-e/commit/1e0b5f9ab5d355122e0610d5be2cc92a875df555))


### Bug Fixes

* **ci:** exclude Tauri GUI crate from test and clippy jobs ([af19683](https://github.com/us/null-e/commit/af1968380fbb3b6b42752c2fa9d44f53f884bb0a))
* **ci:** switch release-please to simple mode for Cargo workspace compatibility ([e0643b5](https://github.com/us/null-e/commit/e0643b576d0601bf426420eb3887107ae17f55cd))

## [0.3.0](https://github.com/us/null-e/compare/v0.2.0...v0.3.0) (2026-03-25)


### ⚠ BREAKING CHANGES

* config directory changed from ~/.config/devsweep to ~/.config/null-e

### Features

* add null-e text next to robot mascot ([1846f3a](https://github.com/us/null-e/commit/1846f3a110755a6a01f09d7e543342c0207a05a7))
* added multi-platform distribution support ([dfeb712](https://github.com/us/null-e/commit/dfeb7121b009cfc06600f6345f4b4531055ddecc))
* added tui ([7feb994](https://github.com/us/null-e/commit/7feb9942ec6c05bfcd1ab9f8130d940200140341))
* apply Minimal Jekyll theme ([dbc0ee3](https://github.com/us/null-e/commit/dbc0ee3741624022b8dc79898e4ba5271e384c0c))
* apply project template (release-please, Makefile, pre-commit, README format) ([9e15641](https://github.com/us/null-e/commit/9e1564165962cb321cd1b5769daeed1c3059940b))
* complete project review, fix 15 bugs, improve detection coverage ([6ef1bf6](https://github.com/us/null-e/commit/6ef1bf6c3cde9660969f41fb64c0fbff1cdea67f))
* major TUI improvements and new cleaners ([1754edc](https://github.com/us/null-e/commit/1754edc2aaf83b3ffe9756e74a3753df34c13556))
* modernize Jekyll site with professional design ([02933ce](https://github.com/us/null-e/commit/02933ce92ba772c2981c3450912daba775b51d0e))


### Bug Fixes

* add Jekyll layout templates for proper rendering ([bbb4bbb](https://github.com/us/null-e/commit/bbb4bbbca6d86f47719732fe3ff13800ab11cd08))
* gate platform-specific imports with cfg attributes ([d05bee6](https://github.com/us/null-e/commit/d05bee6c65ef918180cb73a0f26b99c7278a2c1e))
* hardcode footer bg, increase text opacity, add article padding ([8abeb6c](https://github.com/us/null-e/commit/8abeb6cd575cbec36650c8cb4e4a12605ac0436b))
* keep CleanableItem and Result imports available for non-macOS stub ([1c5ec43](https://github.com/us/null-e/commit/1c5ec4329f56e77bf1618217041094743ce3f870))
* limit ASCII art width to prevent overflow ([60e02e7](https://github.com/us/null-e/commit/60e02e7720a209ca1c1379d06cb36b56b925f379))
* move ASCII art outside center div to prevent alignment issues ([6c4db7d](https://github.com/us/null-e/commit/6c4db7db9bc1006cae28cb651fc52e8213d9bdc4))
* prevent markdown headings inside code blocks ([69a1d91](https://github.com/us/null-e/commit/69a1d9174efe018911c3766fbab91a86a09a53be))
* reduce keywords to 5 for crates.io ([664ebbd](https://github.com/us/null-e/commit/664ebbd0d413fd116eece77bdb98e6e1433fb54f))
* replace ASCII art with text to prevent overflow ([9be3dd2](https://github.com/us/null-e/commit/9be3dd28dee2636f7abf58bcbc37b80f532ed5db))
* restore ASCII art with ultra-small font size (7px) to prevent overflow ([16166df](https://github.com/us/null-e/commit/16166df9c3c10199b161555d8c77565ab5530b78))
* sidebar overflow and blog section improvements ([909cee9](https://github.com/us/null-e/commit/909cee9555d59828e6c5e8c3eca4d423d5006c05))
* update footer to TMLS design with brand + meta layout ([974445c](https://github.com/us/null-e/commit/974445c42ce3ab413cdbd34d9e335cdaa170e501))

## Changelog
