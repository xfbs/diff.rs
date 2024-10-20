# diff.rs

[![ci status](https://gitlab.com/xfbs/diff.rs/badges/master/pipeline.svg)](https://gitlab.com/xfbs/diff.rs/-/pipelines)

Web application to view the difference between Rust crate versions. It is
deployed at [diff.rs](https://diff.rs).

## How it works

It uses [Yew](https://yew.rs) as the reactive frontend framework, and [Tailwind
CSS](https://tailwindcss.com) for the styling.

The code in this repository compiles into a WebAssembly binary that runs in the
browser. Since it only talks to the [crates.io](https://crates.io) API, it does
not need any backend and can be hosted statically.

The [Architecture](docs/architecture.md) document explains the details of how
it works.

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
`wasm-opt` for further size savings. You might see a large speedup from running
it this way.

```
trunk build --release
```

## How it is deployed

It is currently hosted by GitLab Pages using [this CI config](.gitlab-ci.yml).
It serves the application statically, and uses both gzip and brotli compression
for smaller assets and faster loading times.

The deployed version of it has privacy-preserving (cookie-less) analytics
enabled to help me measure how many people are actively using it. If you have
an adblocker, this is likely blocked by it. You can see the data
[here][analytics]

See [Deployment](docs/deployment.md) for more information on how this works.

## How to contribute

Contributions to `diff.rs` are more than welcome. Feel free to check out the
open [issues][] for inspiration.

See [Contributing](docs/contributing.md) for more information on how to
contribute.

## License

MIT, see [LICENSE.md](LICENSE.md).

[analytics]: https://counter.dev/dashboard.html?user=xfbs&token=4kPlix1Li7w%3D
[issues]: https://github.com/xfbs/diff.rs/issues
