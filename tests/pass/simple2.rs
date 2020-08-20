use named::named;

#[named(defaults(a = false, b = false))]
fn or(a: bool, b: bool) -> bool {
    a || b
}

fn main() {
    // 0 specified
    assert!(!or!());

    // 1 specified
    assert!(or!(a = true));
    assert!(or!(b = true));
    assert!(!or!(a = false));
    assert!(!or!(b = false));

    // 2 specified
    assert!(or!(a = true, b = false));
    assert!(or!(a = false, b = true));
    assert!(or!(a = true, b = true));
    assert!(!or!(a = false, b = false));
}
