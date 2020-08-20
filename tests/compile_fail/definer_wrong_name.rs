use named::named;

#[named(defaults(a = false, d = false, c = false))]
fn or(a: bool, b: bool, c: bool) -> bool {
    a || b || c
}

fn main() {
    let _ = or!();
}
