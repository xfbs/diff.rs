use vergen::EmitBuilder;

fn main() {
    EmitBuilder::builder().all_git().emit().unwrap();
}
