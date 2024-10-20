# Contributing

This document explains how you can contribute to `diff.rs`. It outlines some
of the processes for contributing, what you should keep in mind for a successful
contribution.

## General

The scope of `diff.rs` is to be a useful tool to help in:

- Reviewing what has changed in between versions of crates published on
  [crates.io][], both for auditing purposes, for discovering new features and
  for upgrading uses of the crates.
- Reviewing the contents of crates which are published on [crates.io][],
  similar to how [docs.rs][] is a useful tool in viewing crate documentation.
- Viewing differences between crate versions published on [crates.io][] and in
  their repository.

If you have an idea for a feature or you find a bug, feel free to open up an
[issue][issues]. That way, it can be discussed and tracked. In general, all new
feature suggestions are welcome as long as they are in scope for the project,
and can be reasonably maintained.

In general, anyone is welcome to contribute features and bugfixes as [pull
requests][pulls]. For large-scale changes, it usually makes sense to create an
issue with the idea first to get some feedback and align on the implementation.

## Dependencies

In general, use whatever gets the job done. Because `diff.rs` is compiled to
WebAssembly, it tends to be easiest to use dependencies which are implemented
in Rust and don't link to native code. For example, the `syntect` crate has
several backends for its regex implementation, and `diff.rs` uses the
`fancy-regex` crate which is written in pure Rust rather than the default
`onig`, which links to a C library.

One consideration to make when adding dependencies is code size. Large
dependencies might bloat the resulting WebAssembly output and slow down loading
of the application. `diff.rs` already uses some techniques to mitigate this,
including enabling [link-time optimizations][lto], using `wasm-opt` to slim
down the blob, and serving it pre-compressed.

If you add a dependency, you can see the impact it has on the resulting
output by running the `build` task, which will build it in release mode and
compress the files in the same way as in CI:

```bash
$ just build
$ ls -lah dist/*.wasm*
-rw-r--r-- 1 user user 3.4M Oct 20 12:12 dist/diff-rs-60a0fb1e19269ab7_bg.wasm
-rw-r--r-- 1 user user 1.1M Oct 20 12:12 dist/diff-rs-60a0fb1e19269ab7_bg.wasm.br
-rw-r--r-- 1 user user 1.5M Oct 20 12:12 dist/diff-rs-60a0fb1e19269ab7_bg.wasm.gz
```

Our aim is to keep the compressed assets (specifically the brotli) one around
1MB in size. But this is always a tradeoff, adding a new feature which adds
useful functionality might be worth it even if it results in larger sizes.

## Styling

For styling, `diff.rs` uses [Tailwind CSS][tailwind]. This exports CSS utility
classes that can be used to add styling to HTML documents. The Tailwind CSS
generator is automatically run by [Trunk][trunk], the build tool `diff.rs`
uses.

For prototyping, these classes are typically directly added to HTML elements,
like this:

```html
<div class="flex flex-row gap-2">
    <div class="border rounded p-2">Entry 1</div>
    <div class="border rounded p-2">Entry 2</div>
    <div class="border rounded p-2">Entry 3</div>
</div>
```

Once the result looks good, custom CSS classes should be created in the
`src/tailwind.css` file. Doing this allows separating the styling from
the content, and allows for adding useful comments for the styling.

For example, this would involve replacing the Tailwind utility classes
with some custom ones:

```html
<div class="my-component">
    <div class="entry">Entry 1</div>
    <div class="entry">Entry 2</div>
    <div class="entry">Entry 3</div>
</div>
```

Then, the utility classes are applied to the style of the custom elements:

```css
.my-components {
    /* render children as a flex-row with some gaps */
    @apply flex flex-row gap-2;
}

.my-component .entry {
    /* adds a rounded border to entries */
    @apply border rounded;
    /* adds some padding to the inside */
    @apply p-2;
}
```


## Documentation

If you implement a new feature, you should try to document it. Documentation is
not a requirement, but helps in the maintenance of contributed code.  This
includes:

- Documentation comments. For example, components should have a documentation
  comment explaining their purpose.
- Code comments, explaining what is being done.
- Updating `docs/architecture.md` if the contribution changes the high-level
  architecture of `dif.rs`.

## Testing

Currently, `diff.rs` has a limited amount of unit tests. Ideally, there should
be more tests to avoid regressions. However, some things are hard to test, such
as user-interfaces, especially across browsers. For this reason, some amount of
manual testing is performed.

Before you submit a patch, you should run the tests. A useful amount of tests
are captured in the `ci` job of the Justfile. This will test formatting, code
style, run the existing unit tests and perform a build.

    just ci

For the actual application, there are currently no automated tests in place.
The current testing procedure is to open up a local build of `diff.rs` in
different browsers, and verifying that everything functions as expected.
Browsers that should work are:

- Firefox
- Chromium
- Safari

If possible, use the *Response Design Mode* of each browser to verify that the
side looks good on both small, medium and large screens.

It is not required to support all browsers, for example old versions of IE.  It
is more important that the implementation is simple and works well.

## Commits

We don't have concrete rules on how to structure commits or write commit messages.
The only advice is to:

- Try to keep commits atomic, meaning that they change one thing (implement one
  feature). This is useful for bisection.
- Every commit should compile and pass all tests, this is useful for bisection.
- Try to keep commit messages descriptive. One sentence is sufficient.
- Try to keep a [linear history][git-linear]. We use rebasing for pull requests
  and avoid merge commits.

[pulls]: https://github.com/xfbs/diff.rs/pulls
[issues]: https://github.com/xfbs/diff.rs/issues
[crates.io]: https://crates.io/
[docs.rs]: https://docs.rs/
[git-linear]: https://www.bitsnbites.eu/a-tidy-linear-git-history/
[tailwind]: https://tailwindcss.com/docs/
[trunk]: https://rustrs.dev/
[lto]: https://nnethercote.github.io/perf-book/build-configuration.html#link-time-optimization
