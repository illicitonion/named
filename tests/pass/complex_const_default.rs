use named::named;

pub struct D {
    pub value: u8,
}

const DEFAULT: D = D { value: 1 };

#[named(defaults(a = DEFAULT.value))]
fn is_one(a: u8) -> bool {
    a == 1
}

fn main() {
    assert!(is_one!());

    assert!(is_one!(a = 1));

    assert!(!is_one!(a = 2));
}
