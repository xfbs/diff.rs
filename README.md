# diff.rs

Web application to view the difference between crate versions. It is deployed
at [diff.rs](https://diff.rs).

## How it works

The code in this repository compiles into a WebAssembly binary that runs in the
browser.  Since it only talks to the [crates.io](https://crates.io) API, it
does not need any backend and can be hosted statically.

It uses [Yew](https://yew.rs) as the reactive frontend framework, and
[blueprint.js](https://blueprintjs.com) with
[yewprint](https://docs.rs/yewprint) for many of the components.

To render a diff, it uses [reqwest](https://docs.rs/reqwest) to make a request
to the [crates.io](https://crates.io) API in order to fetch crate metadata.
This is a JSON structure that is parsed into a `CrateResponse` using
[serde](https://docs.rs/serde) and [serde_json](https://docs.rs/serde_json).

Using that response, the code will resolve the versions that are in the URL by
looking them up in the `versions` field of that response. If they exist, the
code then performs another request to fetch the crate sources.  These are
gzip-compressed tar balls, which are decompressed using
[flate2](https://docs.rs/flate2) and extracted in-memory using
[tar](https://docs.rs/tar). 

Finally, the code uses [similar](https://docs.rs/simiar) to generate a diff and
render it in the browser.

## How to launch it

You need a recent version of Rust, which is most easily installed with
[rustup.rs](https://rustup.rs).

You need the WebAssembly build target for Rust, which you can install like
this:

```
rustup target add wasm32-unknown-unknown
```

You need [Trunk](https://trunkrs.dev/), which is a tool that helps to build
Rust WebAssembly applications. You can install it like this:

```
cargo install trunk
```

You can then use trunk to launch a local server with hot-reloading for
development purposes:

```
trunk serve
```

You can also use it to build an optimized release version.

```
trunk build --release
```

## How it is deployed

It is currently hosted by GitLab Pages using [this tiny CI
config](.gitlab-ci.yml).

## License

MIT, see [LICENSE.md](LICENSE.md).
