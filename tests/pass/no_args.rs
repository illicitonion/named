use named::named;

#[named]
fn always_true() -> bool {
    true
}

fn main() {
    assert!(always_true!());
}
