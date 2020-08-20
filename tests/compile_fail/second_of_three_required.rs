use named::named;

#[named(defaults(a = 1))]
fn foo(a: u8, b: u8, c: u8) -> String {
    format!("a=[{}], b=[{}], c=[{}]", a, b, c)
}

fn main() {
    let _ = foo!(c = 8);
}
