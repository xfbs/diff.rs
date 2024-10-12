use vergen_gitcl::{Emitter, GitclBuilder};

/// This build script will query your `git` executable to fetch the current commit hash, and make
/// it available to the application using an environment variable. This is used to show the commit
/// hash that diff.rs was built with in the footer.
fn main() {
    let gitcl = GitclBuilder::all_git().unwrap();
    Emitter::default()
        .add_instructions(&gitcl)
        .unwrap()
        .emit()
        .unwrap();
}
