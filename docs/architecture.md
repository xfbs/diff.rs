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
  definitions. The router will map every route to a view.
- `src/views/` contains views, these are the root components for entire pages.
  There is one module per view. Every module exports a single view, but can
  also contain private components which are only used in that particular view.
- `src/components/` contains components which are re-used across views.
- `src/data.rs` contains the types used to communicate with the crates.io API,
  the crate source parsing, and the diffing logic.
- `src/cache.rs` contains the in-memory caches for crate metadata and crate
  sources, which avoid repeat network requests during a session.
- `src/syntax.rs` contains syntax highlighting helpers built on top of
  [syntect](https://docs.rs/syntect).
- `src/version.rs` contains the `VersionId` type, which can represent an exact
  version, a version requirement, or a named version such as `latest`.
- `src/tailwind.css` contains styles for the components and views.
- `index.html` contains the skeleton and metadata for Trunk for which assets to
  build and bundle.

See also [Contributing](contributing.md) for more information on the structure.

## Fetching Crate Info

To render a diff, it uses [gloo-net](https://docs.rs/gloo-net) to make a
request to the [crates.io](https://crates.io) API in order to fetch crate
metadata. This is a JSON structure that is parsed into a `CrateResponse` using
[serde](https://docs.rs/serde) and [serde_json](https://docs.rs/serde_json).

## Diffing Crates

Using that response, the code will resolve the versions that are in the URL by
looking them up in the `versions` field of that response. If they exist, the
code then performs another request to fetch the crate sources. These are
gzip-compressed tar balls, which are decompressed using
[flate2](https://docs.rs/flate2) and extracted in-memory using
[tar](https://docs.rs/tar).

Before the archive is extracted, the SHA-256 hash of the downloaded tarball is
verified against the checksum returned by the crates.io API. This ensures that
what gets diffed is exactly what crates.io has on record. Non-UTF-8 paths and
paths outside the expected `{crate}-{version}/` prefix inside the archive are
rejected, which prevents files from being silently hidden from the UI.

Finally, the code uses [similar](https://docs.rs/similar) to generate a diff
and render it in the browser. It uses [syntect](https://docs.rs/syntect) for
syntax highlighting.

## Caching

To avoid repeated network requests for the same data during a session,
`diff.rs` maintains two in-memory caches in `src/cache.rs`:

- `CRATE_RESPONSE_CACHE` stores parsed `CrateResponse` values keyed by crate
  name.
- `CRATE_SOURCE_CACHE` stores parsed `CrateSource` values keyed by
  `(crate_name, version)`.

Both caches are global singletons held behind a `Mutex`. They live for the
lifetime of the page — there is no persistence across reloads.

## Diffing Published Crates Against Their Repositories

In addition to diffing two published crate versions, `diff.rs` can also diff a
published crate version against the source in its repository at the commit it
was published from. The commit SHA is read from the `.cargo_vcs_info.json`
file that Cargo embeds in every published crate. The repository tarball is
then downloaded from GitHub or GitLab. Because those hosts do not send
permissive CORS headers, the request is routed through `corsproxy.io`.

[yew]: https://yew.rs
