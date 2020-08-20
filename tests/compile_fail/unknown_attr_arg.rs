use named::named;

#[named(defaults(a = false, c = false))]
fn or(a: bool, b: bool) -> bool {
    a || b
}

fn main() {}
