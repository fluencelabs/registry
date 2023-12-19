# Changelog

## [0.9.2](https://github.com/fluencelabs/registry/compare/registry-v0.9.1...registry-v0.9.2) (2023-12-19)


### Features

* update marine sdk's, sqlite conector and config ([#309](https://github.com/fluencelabs/registry/issues/309)) ([863ae55](https://github.com/fluencelabs/registry/commit/863ae55f35bbe5452b636c064f9f8b377bb10ee8))


### Bug Fixes

* **ci:** setup fcli in release step ([#305](https://github.com/fluencelabs/registry/issues/305)) ([7b89267](https://github.com/fluencelabs/registry/commit/7b892678b1003bcf0c0fc834b7b49ceb2172e388))
* **deps:** update dependency @fluencelabs/aqua-lib to v0.8.1 ([#249](https://github.com/fluencelabs/registry/issues/249)) ([66a42f7](https://github.com/fluencelabs/registry/commit/66a42f7b935e82af9133e2d5bc2c864cb4296e2f))
* **deps:** update dependency @fluencelabs/aqua-lib to v0.8.2 ([#308](https://github.com/fluencelabs/registry/issues/308)) ([c207f7f](https://github.com/fluencelabs/registry/commit/c207f7fa549702c45dd8f25d0f97d95944472e6e))
* **deps:** update dependency @fluencelabs/trust-graph to v0.4.7 ([#257](https://github.com/fluencelabs/registry/issues/257)) ([a6aeeea](https://github.com/fluencelabs/registry/commit/a6aeeea3f5eb4f06a99ec272e0f5d3b4b0a2a8a7))

## [0.9.1](https://github.com/fluencelabs/registry/compare/registry-v0.9.0...registry-v0.9.1) (2023-12-06)


### Features

* use non-npm Fluence CLI ([#302](https://github.com/fluencelabs/registry/issues/302)) ([d77fd12](https://github.com/fluencelabs/registry/commit/d77fd12b4dfe2d57ae3e35f729e35e2f6ad1c63c))


### Bug Fixes

* **deps:** update dependency @fluencelabs/cli to v0.13.0 ([#290](https://github.com/fluencelabs/registry/issues/290)) ([2a440a8](https://github.com/fluencelabs/registry/commit/2a440a8b1ff8aa922bd2faa982b8b75c9beb3bc7))
* **deps:** update rust crate marine-rs-sdk-test to v0.11.1 ([#292](https://github.com/fluencelabs/registry/issues/292)) ([2405f41](https://github.com/fluencelabs/registry/commit/2405f41702543d1ff70620923787a6a7621cc7d5))
* remove binary import ([#304](https://github.com/fluencelabs/registry/issues/304)) ([c160475](https://github.com/fluencelabs/registry/commit/c16047515751f1400cb1f7231abcc83e2f6bcf4f))

## [0.9.0](https://github.com/fluencelabs/registry/compare/registry-v0.8.8...registry-v0.9.0) (2023-11-22)


### ⚠ BREAKING CHANGES

* **subnetwork:** deprecate registry-based subnets [NET-633] ([#283](https://github.com/fluencelabs/registry/issues/283))

### Features

* **subnetwork:** deprecate registry-based subnets [NET-633] ([#283](https://github.com/fluencelabs/registry/issues/283)) ([81f15d4](https://github.com/fluencelabs/registry/commit/81f15d4eb74b730fca331f1ea4ef6b960a02f9c8))

## [0.8.8](https://github.com/fluencelabs/registry/compare/registry-v0.8.7...registry-v0.8.8) (2023-11-07)


### Features

* prepare cli update ([#270](https://github.com/fluencelabs/registry/issues/270)) ([2c29fea](https://github.com/fluencelabs/registry/commit/2c29fea09808e2f98c4f58a10a1587aa5a571ad0))
* **registry:** Use streams instead of options [LNG-277]  ([#282](https://github.com/fluencelabs/registry/issues/282)) ([19f5d47](https://github.com/fluencelabs/registry/commit/19f5d47add949f62085a022a01b84c83d3fc0389))


### Bug Fixes

* **ci:** use unstable nox image ([#255](https://github.com/fluencelabs/registry/issues/255)) ([257516e](https://github.com/fluencelabs/registry/commit/257516e74ff78807f78a7570ccc9e2d685af48f9))
* **deps:** unlock and update rust crate serde to 1.0.188 ([#273](https://github.com/fluencelabs/registry/issues/273)) ([4cb1b90](https://github.com/fluencelabs/registry/commit/4cb1b90a95bdc49b87b1dd1336e604cc71444de3))
* **deps:** Update cli to 0.11.0 ([#272](https://github.com/fluencelabs/registry/issues/272)) ([0ac1b76](https://github.com/fluencelabs/registry/commit/0ac1b76fe1c0635bfa5cf1105ffaf899db36b300))
* **deps:** update dependency @fluencelabs/cli ([#276](https://github.com/fluencelabs/registry/issues/276)) ([2259425](https://github.com/fluencelabs/registry/commit/22594259767fbd5be59904eab080d74733e7ea3e))
* **deps:** update dependency @fluencelabs/cli to v0.6.0 ([#238](https://github.com/fluencelabs/registry/issues/238)) ([be441e8](https://github.com/fluencelabs/registry/commit/be441e86cbc07a51636edfd07ec0fc80933b31cf))
* **deps:** update dependency @fluencelabs/fluence-network-environment to v1.1.2 ([#277](https://github.com/fluencelabs/registry/issues/277)) ([8ff086a](https://github.com/fluencelabs/registry/commit/8ff086a206d37edaeebe986661b626277e456d95))
* **deps:** update marine things ([#278](https://github.com/fluencelabs/registry/issues/278)) ([1f44cdc](https://github.com/fluencelabs/registry/commit/1f44cdc3b1188ef9daaba33a73ee85980c0c8bc6))
* **deps:** update rust crate marine-rs-sdk to v0.9.0 ([#265](https://github.com/fluencelabs/registry/issues/265)) ([9b4142d](https://github.com/fluencelabs/registry/commit/9b4142dc951414270f5a76b0519aa749c8835eb6))

## [0.8.7](https://github.com/fluencelabs/registry/compare/registry-v0.8.6...registry-v0.8.7) (2023-06-20)


### Features

* add distro crate [fixes NET-462] ([#233](https://github.com/fluencelabs/registry/issues/233)) ([5acf1d2](https://github.com/fluencelabs/registry/commit/5acf1d230b92f6b0784314b0926b6f6c2e195307))
* Migrate Registry to spell ([#247](https://github.com/fluencelabs/registry/issues/247)) ([990b588](https://github.com/fluencelabs/registry/commit/990b588b75857d2f61b76d89999a2c1f09f861f8))
* update to node 18 ([a08ee16](https://github.com/fluencelabs/registry/commit/a08ee16ff9dc402e1388e22c57324ca975c1a94d))

## [0.8.6](https://github.com/fluencelabs/registry/compare/registry-v0.8.5...registry-v0.8.6) (2023-05-19)


### Features

* **parser:** Fix indentation ([#241](https://github.com/fluencelabs/registry/issues/241)) ([d96f5a4](https://github.com/fluencelabs/registry/commit/d96f5a4a0da7288ef6895c270fe207ea9a9f102d))

## [0.8.5](https://github.com/fluencelabs/registry/compare/registry-v0.8.4...registry-v0.8.5) (2023-05-08)


### Features

* **builtin-package:** use new blueprint ([#234](https://github.com/fluencelabs/registry/issues/234)) ([061cf2f](https://github.com/fluencelabs/registry/commit/061cf2f8186192c39946628e21e466323dc31a33))

## [0.8.4](https://github.com/fluencelabs/registry/compare/registry-v0.8.3...registry-v0.8.4) (2023-04-19)


### Features

* update aqua-lib and trust-graph versions ([#229](https://github.com/fluencelabs/registry/issues/229)) ([5e460e3](https://github.com/fluencelabs/registry/commit/5e460e3e2429df909d034193fedf2876f86b18a8))


### Bug Fixes

* **deps:** pin dependencies ([#198](https://github.com/fluencelabs/registry/issues/198)) ([e66457c](https://github.com/fluencelabs/registry/commit/e66457c0ff696330717e58e3ebb4120709281202))
* **deps:** update dependency @fluencelabs/fluence-network-environment to v1.0.14 ([#195](https://github.com/fluencelabs/registry/issues/195)) ([204af45](https://github.com/fluencelabs/registry/commit/204af450001cd6e1ed587111fcc452d41d56a705))

## [0.8.3](https://github.com/fluencelabs/registry/compare/registry-v0.8.2...registry-v0.8.3) (2023-04-06)


### Features

* **sqlite:** bump to v0.18.1 ([#218](https://github.com/fluencelabs/registry/issues/218)) ([4fd0895](https://github.com/fluencelabs/registry/commit/4fd0895ab8415b60eacb34e0a627e9d6d5b5fe2c))

## [0.8.2](https://github.com/fluencelabs/registry/compare/registry-v0.8.1...registry-v0.8.2) (2023-03-08)


### Bug Fixes

* **deps:** Update sqlite to 0.8.0 ([#205](https://github.com/fluencelabs/registry/issues/205)) ([d27f232](https://github.com/fluencelabs/registry/commit/d27f232fb44629b18fa45e45b7c33e332f5817fd))

## [0.8.1](https://github.com/fluencelabs/registry/compare/registry-v0.8.0...registry-v0.8.1) (2023-02-24)


### Bug Fixes

* **subnet:** add on HOST_PEER_ID in resolveSubnetwork ([#202](https://github.com/fluencelabs/registry/issues/202)) ([3960180](https://github.com/fluencelabs/registry/commit/3960180246471a78bacf5fa65152a52fb3d4ddf2))

## [0.8.0](https://github.com/fluencelabs/registry/compare/registry-v0.7.1...registry-v0.8.0) (2023-02-24)


### ⚠ BREAKING CHANGES

* **storage:** bump SQLite module to 0.18.0 ([#200](https://github.com/fluencelabs/registry/issues/200))

### Bug Fixes

* **storage:** bump SQLite module to 0.18.0 ([#200](https://github.com/fluencelabs/registry/issues/200)) ([f671c8a](https://github.com/fluencelabs/registry/commit/f671c8ac1514a11331ae871a7e126f1e908214f6))

## [0.7.1](https://github.com/fluencelabs/registry/compare/registry-v0.7.0...registry-v0.7.1) (2023-02-20)


### Features

* **deals:** register and resolve workers ([#197](https://github.com/fluencelabs/registry/issues/197)) ([8d49211](https://github.com/fluencelabs/registry/commit/8d492113f17ec7add582f7f2d9575fc48b5325dc))
* **tests:** Run tests using fluence cli [fixes DXJ-225] ([#165](https://github.com/fluencelabs/registry/issues/165)) ([269373f](https://github.com/fluencelabs/registry/commit/269373f0ea904c572cffa51b8d49a248822c7ff1))


### Bug Fixes

* run all tests with different secret keys [fixes DXJ-242] ([#187](https://github.com/fluencelabs/registry/issues/187)) ([9b5cfbd](https://github.com/fluencelabs/registry/commit/9b5cfbd987259a890933e516e8ec2fee58e149d8))
* **tests:** fix registry aqua tests [fixes DXJ-235] ([#178](https://github.com/fluencelabs/registry/issues/178)) ([9981043](https://github.com/fluencelabs/registry/commit/9981043448fa3a9d64353ab763f9985245a6dff0))
