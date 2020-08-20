use named::named;

const DEFAULT: u8 = 1;

#[named(defaults(a = DEFAULT))]
fn is_one(a: u8) -> bool {
    a == 1
}

fn main() {
    assert!(is_one!());

    assert!(is_one!(a = 1));

    assert!(!is_one!(a = 2));
}
