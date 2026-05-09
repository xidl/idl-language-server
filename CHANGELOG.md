# Changelog

## [0.25.0](https://github.com/xidl/idl-language-server/compare/v0.24.0...v0.25.0) (2026-05-09)


### Features

* bump tokio from 1.52.2 to 1.52.3 ([#63](https://github.com/xidl/idl-language-server/issues/63)) ([8fb6efa](https://github.com/xidl/idl-language-server/commit/8fb6efa9813d3d2809a9ddd99b0f24d9def69e06))
* bump xidl-parser from 0.55.0 to 0.56.0 ([#65](https://github.com/xidl/idl-language-server/issues/65)) ([44c6928](https://github.com/xidl/idl-language-server/commit/44c6928b86b11741b968895fd8c546e87269fa2f))
* bump xidlc from 0.55.0 to 0.56.0 ([#64](https://github.com/xidl/idl-language-server/issues/64)) ([fd3610f](https://github.com/xidl/idl-language-server/commit/fd3610fac28cc499201be5412c01a0bd6c9e0222))

## [0.24.0](https://github.com/xidl/idl-language-server/compare/v0.23.1...v0.24.0) (2026-05-08)


### Features

* bump tower-http from 0.6.9 to 0.6.10 ([#59](https://github.com/xidl/idl-language-server/issues/59)) ([1900768](https://github.com/xidl/idl-language-server/commit/1900768835514191f0ef1fb75068b6bc81f9b863))
* bump xidl-parser from 0.54.0 to 0.55.0 ([#60](https://github.com/xidl/idl-language-server/issues/60)) ([e4696e8](https://github.com/xidl/idl-language-server/commit/e4696e89fb9615a6900a5d1656e42688734fe714))
* bump xidlc from 0.54.0 to 0.55.0 ([#58](https://github.com/xidl/idl-language-server/issues/58)) ([2339ed2](https://github.com/xidl/idl-language-server/commit/2339ed25f145e70e5091ebfaae0f327514bdbc45))
* improve error handling with context and add logging for key steps ([5a1c1c4](https://github.com/xidl/idl-language-server/commit/5a1c1c4500dc252627b8369453bf843afcd55282))


### Bug Fixes

* **release:** fix git lfs ([79a8de7](https://github.com/xidl/idl-language-server/commit/79a8de735cedbc7b312ab356f4e3c6e23ab01a89))

## [0.23.1](https://github.com/xidl/idl-language-server/compare/v0.23.0...v0.23.1) (2026-05-07)


### Bug Fixes

* revert "feat: add preview generated code feature" ([7bf45c4](https://github.com/xidl/idl-language-server/commit/7bf45c432ec6abf35a36ab5d8bccc2d6938cff4c))

## [0.23.0](https://github.com/xidl/idl-language-server/compare/v0.22.0...v0.23.0) (2026-05-07)


### Features

* add preview generated code feature ([f2783d6](https://github.com/xidl/idl-language-server/commit/f2783d65f1827c5d106c9e714b343a8b1ede729d))
* bump tower-http from 0.6.8 to 0.6.9 ([#52](https://github.com/xidl/idl-language-server/issues/52)) ([8935d6a](https://github.com/xidl/idl-language-server/commit/8935d6a0cb775cce9036237f401567ccbb7cbf92))
* **http_client:** make openapi regeneration command configurable ([1a81576](https://github.com/xidl/idl-language-server/commit/1a815761ec3f223a931c761413892d23a72e1116))
* make xidlc path configurable ([ca6f9e0](https://github.com/xidl/idl-language-server/commit/ca6f9e010090a1621d680c842c02f8084ba16e2d))


### Bug Fixes

* **openapi:** fix openapi generate ([cda5d5b](https://github.com/xidl/idl-language-server/commit/cda5d5bf971c2d7f34217e32b57cee818aebb688))

## [0.22.0](https://github.com/xidl/idl-language-server/compare/v0.21.0...v0.22.0) (2026-05-06)


### Features

* bump tokio from 1.52.1 to 1.52.2 ([#48](https://github.com/xidl/idl-language-server/issues/48)) ([00fd386](https://github.com/xidl/idl-language-server/commit/00fd386be9f8ab28f68f23665b7dd8fd077c7e06))
* bump utoipa from 5.4.0 to 5.5.0 ([#49](https://github.com/xidl/idl-language-server/issues/49)) ([421ac16](https://github.com/xidl/idl-language-server/commit/421ac16b48c26592bd0971166b69ddb8c63fc97e))
* bump xidl-parser from 0.50.2 to 0.53.0 ([#47](https://github.com/xidl/idl-language-server/issues/47)) ([0180a37](https://github.com/xidl/idl-language-server/commit/0180a374889f5a78868ee86e713715a26f0f49d1))
* bump xidl-parser from 0.53.0 to 0.53.1 ([#50](https://github.com/xidl/idl-language-server/issues/50)) ([0983b10](https://github.com/xidl/idl-language-server/commit/0983b10e60356e5fd736bf7d0be72ef825e26fe5))
* bump xidlc from 0.50.2 to 0.53.0 ([#46](https://github.com/xidl/idl-language-server/issues/46)) ([f36f478](https://github.com/xidl/idl-language-server/commit/f36f478552f82652adb9dcd38ba901f3299b7e1d))

## [0.21.0](https://github.com/xidl/idl-language-server/compare/v0.20.0...v0.21.0) (2026-04-30)


### Features

* bump anyhow from 1.0.93 to 1.0.102 ([#41](https://github.com/xidl/idl-language-server/issues/41)) ([4a63d97](https://github.com/xidl/idl-language-server/commit/4a63d970fcf6974783ff4246783a3e1351d41b8f))
* bump axum from 0.8.8 to 0.8.9 ([#42](https://github.com/xidl/idl-language-server/issues/42)) ([d586c99](https://github.com/xidl/idl-language-server/commit/d586c99bb213d7db84362ee72517e5be1f3d41d3))
* rename extension name ([486d65d](https://github.com/xidl/idl-language-server/commit/486d65d25122744a05974a3539ff96a45328e01a))


### Bug Fixes

* fix publish release ([e887b9c](https://github.com/xidl/idl-language-server/commit/e887b9c66eaad97fefd5f64af36c12eba9fea19f))

## [0.20.0](https://github.com/xidl/idl-language-server/compare/v0.19.0...v0.20.0) (2026-04-29)


### Features

* bump env_logger from 0.11.5 to 0.11.6 ([#36](https://github.com/xidl/idl-language-server/issues/36)) ([a9f6c51](https://github.com/xidl/idl-language-server/commit/a9f6c51500a378dddfdf7390905a6e45fb8be50a))
* bump log from 0.4.22 to 0.4.29 ([#35](https://github.com/xidl/idl-language-server/issues/35)) ([9228de7](https://github.com/xidl/idl-language-server/commit/9228de75adae2f98517b2d962652a96ecd5a4b05))
* bump ropey from 1.6.0 to 1.6.1 ([#34](https://github.com/xidl/idl-language-server/issues/34)) ([6ee0bcc](https://github.com/xidl/idl-language-server/commit/6ee0bccd5e74272c9e9e1a9955f1f666f2a036cd))
* bump tokio from 1.50.0 to 1.52.1 ([#37](https://github.com/xidl/idl-language-server/issues/37)) ([8b6f218](https://github.com/xidl/idl-language-server/commit/8b6f218cf97abf2b5bae7281adcf2ee8e35b9dbc))
* bump xidl-parser from 0.47.0 to 0.50.2 ([#39](https://github.com/xidl/idl-language-server/issues/39)) ([2486239](https://github.com/xidl/idl-language-server/commit/2486239518d3a35234cd24edcdbff23aa9a900a7))
* bump xidlc from 0.47.0 to 0.50.2 ([#38](https://github.com/xidl/idl-language-server/issues/38)) ([42f714b](https://github.com/xidl/idl-language-server/commit/42f714befa7585bbb2b356be9bcce9d098acadd7))

## [0.19.0](https://github.com/xidl/idl-language-server/compare/v0.18.0...v0.19.0) (2026-04-22)


### Features

* bump xidl to 0.47.0 ([da4451e](https://github.com/xidl/idl-language-server/commit/da4451e68da8a8d18a2dc52420e42ed112f2a3bf))

## [0.18.0](https://github.com/xidl/idl-language-server/compare/v0.17.0...v0.18.0) (2026-04-03)


### Features

* bump xidl to 0.34.0 ([f4cac7f](https://github.com/xidl/idl-language-server/commit/f4cac7f1155b18791dfd0cd676e3e932651ddc27))

## [0.17.0](https://github.com/xidl/idl-language-server/compare/v0.16.1...v0.17.0) (2026-04-02)


### Features

* **lsp:** add quickfix for unknown type suggestions ([5463d2a](https://github.com/xidl/idl-language-server/commit/5463d2a675ee7cdc50c7b65b3201ce9f0b14ab4e))

## [0.16.1](https://github.com/xidl/idl-language-server/compare/v0.16.0...v0.16.1) (2026-04-01)


### Bug Fixes

* **diagnostics:** suggest similar types for unknown type warnings ([f0b38fc](https://github.com/xidl/idl-language-server/commit/f0b38fcf301520269d381f930e44359d54ef0338))

## [0.16.0](https://github.com/xidl/idl-language-server/compare/v0.15.0...v0.16.0) (2026-04-01)


### Features

* **diagnostics:** warn on missing local type references ([3d71e08](https://github.com/xidl/idl-language-server/commit/3d71e083186d77628545f58e0bc1db001d546721))

## [0.15.0](https://github.com/xidl/idl-language-server/compare/v0.14.1...v0.15.0) (2026-04-01)


### Features

* **lsp:** update semantic token ([e991723](https://github.com/xidl/idl-language-server/commit/e9917238038ac1833a8a63ce0e42c5894cdabad1))
* **vscode:** enable semanticHighlighting by default ([107c5ea](https://github.com/xidl/idl-language-server/commit/107c5ea965e5e039987f7a3288746576a9ecbe35))

## [0.14.1](https://github.com/xidl/idl-language-server/compare/v0.14.0...v0.14.1) (2026-03-31)


### Bug Fixes

* fix cors problem ([843e8d7](https://github.com/xidl/idl-language-server/commit/843e8d71f6d7c7f4b4a6dfe2925314d30d702f31))

## [0.14.0](https://github.com/xidl/idl-language-server/compare/v0.13.1...v0.14.0) (2026-03-30)


### Features

* add inspect hir and inspect typedast ([c58d59b](https://github.com/xidl/idl-language-server/commit/c58d59b142ca368b1a7ad3c3e1f9b300c30c231f))

## [0.13.1](https://github.com/xidl/idl-language-server/compare/v0.13.0...v0.13.1) (2026-03-24)


### Bug Fixes

* **vscode:** fix icon ([9f22703](https://github.com/xidl/idl-language-server/commit/9f22703ef08d04d60068ce4fe89d6e99ae8c54fd))

## [0.13.0](https://github.com/xidl/idl-language-server/compare/v0.12.0...v0.13.0) (2026-03-24)


### Features

* update readme ([9c3c3bd](https://github.com/xidl/idl-language-server/commit/9c3c3bd323f83ea73aa6282a54e499d440b4e045))

## [0.12.0](https://github.com/xidl/idl-language-server/compare/v0.11.0...v0.12.0) (2026-03-23)


### Features

* update semantic token ([e7177ac](https://github.com/xidl/idl-language-server/commit/e7177acfe2d14b77689762de16f64b47f63849a2))

## [0.11.0](https://github.com/xidl/idl-language-server/compare/v0.10.0...v0.11.0) (2026-03-23)


### Features

* **lsp:** align semantic token legend ([f91c00b](https://github.com/xidl/idl-language-server/commit/f91c00ba17e88c5121c6b64825a5cf3efbb4aac1))
* **scalar:** bump to 1.49.3 ([baa10fa](https://github.com/xidl/idl-language-server/commit/baa10fab1f4daf6cb42ca714eb556b7c2875b1ae))

## [0.10.0](https://github.com/xidl/idl-language-server/compare/v0.9.0...v0.10.0) (2026-03-23)


### Features

* bump deps ([9efc9e3](https://github.com/xidl/idl-language-server/commit/9efc9e3deac900600e349611d8245f46ea976c2e))
* bump nodejs deps ([89beaaf](https://github.com/xidl/idl-language-server/commit/89beaaf4b971b075ae7d36c3c5a63641d1322e98))

## [0.9.0](https://github.com/xidl/idl-language-server/compare/v0.8.0...v0.9.0) (2026-03-22)


### Features

* only show hover and code actin, code lens on interface ([2ab54a7](https://github.com/xidl/idl-language-server/commit/2ab54a7cc06f262a7ee9cc5305ac54ff9e537b24))
* refresn code action ([50b9058](https://github.com/xidl/idl-language-server/commit/50b90585b37371fe425eb0557f7aa87faa71a64f))

## [0.8.0](https://github.com/xidl/idl-language-server/compare/v0.7.0...v0.8.0) (2026-03-21)


### Features

* **ai:** add skill ([5b67bfc](https://github.com/xidl/idl-language-server/commit/5b67bfc7ef864c87afc2ef0bfaea0f304eea03bf))

## [0.7.0](https://github.com/xidl/idl-language-server/compare/v0.6.0...v0.7.0) (2026-03-21)


### Features

* impl rename ([8298dd2](https://github.com/xidl/idl-language-server/commit/8298dd21c006c1e66fe1ee2ba5654529591b9cea))

## [0.6.0](https://github.com/xidl/idl-language-server/compare/v0.5.1...v0.6.0) (2026-03-21)


### Features

* add diagnostic and folding ([19974fb](https://github.com/xidl/idl-language-server/commit/19974fb3487c098b8f3dccf6153e4028e6de3ba5))
* add goto define ([2e0f26d](https://github.com/xidl/idl-language-server/commit/2e0f26d36e2399029616c8bc410a38609f274a47))
* support document symbol ([330d7f7](https://github.com/xidl/idl-language-server/commit/330d7f7f3f9f81826751fde9745f019b3662afae))

## [0.5.1](https://github.com/xidl/idl-language-server/compare/v0.5.0...v0.5.1) (2026-03-21)


### Bug Fixes

* fix ci ([8858183](https://github.com/xidl/idl-language-server/commit/88581832afdef36e7c06b2eac07e96c89adbb50b))

## [0.5.0](https://github.com/xidl/idl-language-server/compare/v0.4.0...v0.5.0) (2026-03-21)


### Features

* remove useless code ([1a38086](https://github.com/xidl/idl-language-server/commit/1a380863bd4a4c4322f2eb58824f88a2a040dcc2))

## [0.4.0](https://github.com/xidl/idl-language-server/compare/v0.3.0...v0.4.0) (2026-03-21)


### Features

* **package:** update ([b5ce047](https://github.com/xidl/idl-language-server/commit/b5ce04789756accb26e2ef6f2a6512f1331dfca7))


### Bug Fixes

* fix language ([b3235e3](https://github.com/xidl/idl-language-server/commit/b3235e3a0b356d4c76399dcbbf48cab710e44ad4))

## [0.3.0](https://github.com/xidl/idl-language-server/compare/v0.2.1...v0.3.0) (2026-03-21)


### Features

* add ci ([e917922](https://github.com/xidl/idl-language-server/commit/e917922c94a8cfa325f3a1384e022d09a7dfb40e))
* add format ([4087d09](https://github.com/xidl/idl-language-server/commit/4087d093c9bdc42633f0ee39320cb12ce770a689))
* init spec ([f9e3e1f](https://github.com/xidl/idl-language-server/commit/f9e3e1f5fe15818769cb6ebd6b0c929f67827ad1))
* udpate token ([ebfdb5e](https://github.com/xidl/idl-language-server/commit/ebfdb5e35a9186758e8b9de494d55a24d9f3f9c2))


### Bug Fixes

* **ci:** fix ([fe0c086](https://github.com/xidl/idl-language-server/commit/fe0c086581f244425020bd654d56251081b80f69))
* fix ci ([2029f2e](https://github.com/xidl/idl-language-server/commit/2029f2e252c0504fa24cdd1808b912030a433e24))

## [0.2.1](https://github.com/xidl/idl-language-server/compare/v0.2.0...v0.2.1) (2026-03-21)


### Bug Fixes

* fix ci ([2029f2e](https://github.com/xidl/idl-language-server/commit/2029f2e252c0504fa24cdd1808b912030a433e24))

## [0.2.0](https://github.com/xidl/idl-language-server/compare/v0.1.0...v0.2.0) (2026-03-21)


### Features

* add ci ([e917922](https://github.com/xidl/idl-language-server/commit/e917922c94a8cfa325f3a1384e022d09a7dfb40e))
* add format ([4087d09](https://github.com/xidl/idl-language-server/commit/4087d093c9bdc42633f0ee39320cb12ce770a689))
* init spec ([f9e3e1f](https://github.com/xidl/idl-language-server/commit/f9e3e1f5fe15818769cb6ebd6b0c929f67827ad1))
* udpate token ([ebfdb5e](https://github.com/xidl/idl-language-server/commit/ebfdb5e35a9186758e8b9de494d55a24d9f3f9c2))
