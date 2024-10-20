# Maintenance

These are maintenance tasks that should be performed regularly. Anyone can
perform these and create a pull request. These actions should only be performed
if they don't break anything, it's okay for `diff.rs` to be a few versions
behind if there are bugs that prevent it from upgrading to newer tool or
dependency versions, unless there are security issues.

## Update Tool Versions

The project uses a number of tools besides the Rust toolchain. Mainly, it uses
Trunk as a build tool, but behind the scenes Trunk downloads and executes
`tailwindcss`, `wasm-opt` and `wasm-bindgen`. The versions of these tools are
defined in the Trunk configuration. When possible, they should be bumped to the
latest versions.

- Update Rust version in `rust-toolchain.toml` to latest
- Update Trunk version to latest in `.gitlab-ci.yml` and update required
  version in `Trunk.toml`
- Update tool versions in `Trunk.toml` to latest (for `tailwindcss`, `wasm-opt`
  and `wasm-bindgen`)

## Update Dependency Versions

The project uses a number of dependencies. When deploying to production, it
always uses the pinned versions in `Cargo.lock`. Occasionally, they should be
updated to get bugfixes and performance enhancements.

- Use `cargo update` to update pinned version dependencies
- Use `cargo outdated` to see if there are newer versions of dependencies and
  manually upgrade

In some cases, it makes sense to use an older version of a dependency. For
example, to avoid needing multiple versions of the same dependency. If
`diff.rs` depends on `yew`, which uses `gloo-0.4.0`, and it itseld needs
`gloo`, it can make sense for it to depend on the same version, otherwise there
will be duplicated versions of `gloo`.

## Testing

You can test if the resulting code works at all by serving it locally with
Trunk.

    trunk serve

You can test if the changes work by running the same checks that the CI runs by
using the `ci` task in the Justfile:

    just ci

You can see the impact on the resulting file sizes by doing a build in the way
the CI does it using the `build` target of the Justfile:

    just build
    ls -lah dist/*.wasm*

If there is no regression in functionality or resulting compressed WebAssembly
code size, the changes should be safe.
