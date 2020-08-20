use named::named;

#[named(defaults(b = false))]
fn or(a: bool, b: bool) -> bool {
    a || b
}

fn main() {
    let _ = or!();
}
