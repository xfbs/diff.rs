# Architecture

## Fetching Crate Info



## Diffing Crates




## Deployment

![Deployment diagram](deployment.svg)

Building is done using [Trunk][trunk]. It uses Cargo under the hood to compile
the code for `wasm32-unknown-unknown`, adds some glue code to launch it in the
browser and links the assets.  Deployment is done using [GitLab
Pages][gitlab-pages]. The CI code uses `gzip` and `brotli` to pre-compress the
files, which makes the resulting WebAssembly blob significantly smaller (at the
time of writing, it makes it go from 3MB to 1MB).

[trunk]: https://trunkrs.dev/
[gitlab-pages]: https://docs.gitlab.com/ee/user/project/pages/
