# Architecture

This document explains the high-level architecture of `diff.rs`. It is intended
as an explainer that is useful for people to quickly get up to speed on how it
works.

## Code Structure

Currently, `diff.rs` is a single-page web application implemented in Rust using
[Yew][yew]. It is structured like this:

- `src/main.rs` is the binary entrypoint. It sets up logging and the Yew rendering.
- `src/lib.rs` is the library entrypoint. It defines the routing and re-exports definitions.
  The router will map every route to a view.
- `src/views/` contains views, these are the root components for entire pages. There is one
  module per view. Every module exports a single view, but can also contain private
  components which are only used in that particular view.
- `src/components/` contains components which are re-used across views.
- `src/tailwind.css` contains styles for the components and views.

## Fetching Crate Info



## Diffing Crates




## Deployment

The workflow for deployment work like this:

- Building the frontend using `trunk build --release`
- Compressing assets using `gzip` and `brotli`
- Adding a `_redirects` file to configure it as a single-page application

![Deployment diagram](deployment.svg)

Building is done using [Trunk][trunk]. It uses Cargo under the hood to compile
the code for `wasm32-unknown-unknown`, adds some glue code to launch it in the
browser and links the assets. Running this includes:

- Running `tailwindcss` to generate the Tailwind CSS styles.
- Using [wasm-bindgen][] to generate bindings for the WebAssembly binary for the browser.
- Using [wasm-opt][] to optimize the resulting WebAssembly binary.
- Including the assets and hashing them for Sub-Resource Integrity and cache busting
- Minifying the assets

Deployment is done using [GitLab CI][gitlab-ci]. The
[configuration](../.gitlab-ci.yml) runs Trunk, and uses `gzip` and `brotli` to
pre-compress the files. Using compression makes the resulting WebAssembly blob
and assets significantly smaller, at the time of writing, it makes it go from
3MB to 1MB. This step consists of:

Hosting of the project is provided by [GitLab Pages][gitlab-pages]. The
resulting files of the building and compression are statically hosted by it.
GitLab Pages will serve the precompressed files if the request indicates that
the client supports it. A `_redirects` file is used to tell it to serve the
`index.html` file for every route, to make it work as a single-page
application.

[trunk]: https://trunkrs.dev/
[gitlab-pages]: https://docs.gitlab.com/ee/user/project/pages/
[gitlab-ci]: https://docs.gitlab.com/ee/ci/
[wasm-opt]: https://github.com/WebAssembly/binaryen
[wasm-bindgen]: https://github.com/rustwasm/wasm-bindgen
[yew]: https://yew.rs
