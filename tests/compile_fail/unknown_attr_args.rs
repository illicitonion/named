use named::named;

#[named(defaults(c = false, d = false))]
fn or(a: bool, b: bool) -> bool {
    a || b
}

fn main() {}
