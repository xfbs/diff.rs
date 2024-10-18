# Maintenance

These are maintenance tasks that should be performed regularly. Anyone can
perform these and create a pull request. These actions should only be performed
if they don't break anything, it's okay for `diff.rs` to be a few versions
behind if there are bugs that prevent it from upgrading to newer tool or
dependency versions, unless there are security issues.

## Update Tool Versions

- Update Rust version in `rust-toolchain.toml` to latest
- Update tool versions in `Trunk.toml` to latest
- Update Trunk version to latest in `.gitlab-ci.yml`

## Update Dependency Versions

- Use `cargo update` to update pinned version dependencies
- Use `cargo outdated` to see if there are newer versions of dependencies and manually upgrade

## Testing

To test if the changed work, simply serve the page with `trunk` and test it
locally (in a browser) to see if it still works with the changes.

    trunk serve --release


