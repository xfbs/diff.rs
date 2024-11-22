## About

Web-based tool to view the differences between Rust crate versions.  Enter a
crate name such as `serde` in the search field in the top-right corner to get
started.  The tool allows you to see what changed between different versions of
the same crate, or different versions of different crates.  This allows you to
see, for example, what changed between a crate and its fork.

It is implemented as a web application written in Rust and compiled to
WebAssembly. It uses the [crates.io](https://crates.io/) API to fetch crate
metadata. To diff two crate versions, it downloads them, validates their hash
sums, decompresses them and extracts their source files in-memory. It runs a
diff algorithm over the source files and renders it with syntax highlighting in
you browser.  Source code for this application is available at
[github.com/xfbs/diff.rs](https://github.com/xfbs/diff.rs).

### Acknowledgements

This project was made possible thanks to contributions made by the
following individuals:

- [Alphare](https://github.com/Alphare): contributed support for diffing across crates
- [eth3lbert](https://github.com/eth3lbert): contributed folding for section which did not change
- [mystor](https://github.com/mystor): contributed syntax highlighting and browsing files of a single crate version
- [SwishSwushPow](https://github.com/SwishSwushPow): contributed hiding of unchanged files and folders
- [j-mahapatra](https://github.com/j-mahapatra): helped migrate legacy CSS rules to Tailwind CSS
- [HWienhold](https://github.com/HWienhold): added file filtering and summary page
- [tverghis](https://github.com/tverghis): added folder expansion and collapsing

This list is not exhaustive, check the repository for a full
[list of contributors](https://github.com/xfbs/diff.rs/graphs/contributors)

Additionally, this tool builds on work done by the Rust ecosystem. It would
not be possible without the following crates:

- [Yew](https://crates.io/crates/yew): responsive frontend web framework for Rust,
- [Syntect](https://crates.io/crates/syntect): syntax highlighter,
- [Similar](https://crates.io/crates/similar): diff algorithm implementation,
- [Tar](https://crates.io/crates/tar): reading of Tar archives,
- [Flate2](https://crates.io/crates/flate2): DEFLATE decompression in pure Rust.

This list is not exhaustive, check the repository for a full [list of crates
used](https://github.com/xfbs/diff.rs/blob/master/Cargo.toml).

### Privacy

This tool runs entirely in your browser and has no backend. As such, no data
is logged by the tool itself.

To view and diff crates, it makes requests to the crates.io API, which may log
requests according to its [Privacy
Policy](https://foundation.rust-lang.org/policies/privacy-policy/).

In addition, diff.rs contains some analytics that is used to measure how many
active users it has. This service stores some data in anonymized fashion,
according to the [Data Policy](https://plausible.io/data-policy).  If you use
an adblocker, then analytics will likely be blocked. It does not use cookies,
fingerprinting or any other invasive means to track visitors. You can view the
collected data
[here](https://counter.dev/dashboard.html?user=xfbs&token=4kPlix1Li7w%3D).

### License

Licensed under the
[MIT](https://github.com/xfbs/diff.rs/blob/master/LICENSE.md) license.

