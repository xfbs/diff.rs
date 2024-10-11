use vergen_gitcl::{Emitter, GitclBuilder};

fn main() {
    let gitcl = GitclBuilder::all_git().unwrap();
    Emitter::default()
        .add_instructions(&gitcl)
        .unwrap()
        .emit()
        .unwrap();
}
