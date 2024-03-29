# diff.rs

Web application to view the difference between Rust crate versions. It is
deployed at [diff.rs](https://diff.rs).

## How it works

It uses [Yew](https://yew.rs) as the reactive frontend framework, and
[blueprint.js](https://blueprintjs.com) with
[yewprint](https://docs.rs/yewprint) for many of the components.

The code in this repository compiles into a WebAssembly binary that runs in the
browser. Since it only talks to the [crates.io](https://crates.io) API, it does
not need any backend and can be hosted statically.

To render a diff, it uses [gloo](https://docs.rs/gloo) to make a request to the
[crates.io](https://crates.io) API in order to fetch crate metadata.  This is a
JSON structure that is parsed into a `CrateResponse` using
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

In addition, you will also need [Trunk](https://trunkrs.dev/), which is a tool
that helps to build Rust WebAssembly applications, and the WebAssembly build
target for Rust.  You can install it like this:

```
cargo install trunk
rustup target add wasm32-unknown-unknown
```

You can then use trunk to launch a local server with hot-reloading for
development purposes:

```
trunk serve
```

You can also use it to build an optimized release version. This will place the
build output into the `dist` folder. The `--release` flag enables some
optimization, both building the Rust code in release mode, and running
`wasm-opt` for further size savings.

```
trunk build --release
```

## How it is deployed

It is currently hosted by GitLab Pages using [this tiny CI
config](.gitlab-ci.yml). It serves the application statically, and uses both
gzip and brotli compression for faster loading times.

## License

MIT, see [LICENSE.md](LICENSE.md).
