use named::named;

#[named(defaults(a = false))]
fn or(a: bool, b: bool) -> bool {
    a || b
}

fn main() {
    let _ = or!(a = true);
}
