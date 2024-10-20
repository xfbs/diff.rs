# Architecture

This document explains the high-level architecture of `diff.rs`. It is intended
as an explainer that is useful for people to quickly get up to speed on how it
works.

## Code Structure

Currently, `diff.rs` is a single-page web application implemented in Rust using
[Yew][yew]. It is structured like this:

- `src/main.rs` is the binary entrypoint. It sets up logging and the Yew
  rendering.
- `src/lib.rs` is the library entrypoint. It defines the routing and re-exports
  definitions.  The router will map every route to a view.
- `src/views/` contains views, these are the root components for entire pages.
  There is one module per view. Every module exports a single view, but can
  also contain private components which are only used in that particular view.
- `src/components/` contains components which are re-used across views.
- `src/tailwind.css` contains styles for the components and views.
- `index.html` contains the skeleton and metadata for Trunk for which assets to
  build and bundle.

See also [Contributing](contributing.md) for more information of the structure.

## Fetching Crate Info

To render a diff, it uses [gloo](https://docs.rs/gloo) to make a request to the
[crates.io](https://crates.io) API in order to fetch crate metadata.  This is a
JSON structure that is parsed into a `CrateResponse` using
[serde](https://docs.rs/serde) and [serde_json](https://docs.rs/serde_json).

## Diffing Crates

Using that response, the code will resolve the versions that are in the URL by
looking them up in the `versions` field of that response. If they exist, the
code then performs another request to fetch the crate sources.  These are
gzip-compressed tar balls, which are decompressed using
[flate2](https://docs.rs/flate2) and extracted in-memory using
[tar](https://docs.rs/tar). 

Finally, the code uses [similar](https://docs.rs/simiar) to generate a diff and
render it in the browser. It uses the [syntect](https://docs.rs/syntect) for
syntax highlighting.

[yew]: https://yew.rs
