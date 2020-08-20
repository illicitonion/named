use named::named;

#[named(defaults(c = 3, b = 2, a = 1))]
fn foo(a: u8, b: u8, c: u8) -> String {
    format!("a=[{}], b=[{}], c=[{}]", a, b, c)
}

fn main() {
    // 0 specified
    assert_eq!("a=[1], b=[2], c=[3]", &foo!());

    // 1 specified
    assert_eq!("a=[9], b=[2], c=[3]", &foo!(a = 9));
    assert_eq!("a=[1], b=[8], c=[3]", &foo!(b = 8));
    assert_eq!("a=[1], b=[2], c=[7]", &foo!(c = 7));

    // 2 specified
    assert_eq!("a=[4], b=[5], c=[3]", &foo!(a = 4, b = 5));
    assert_eq!("a=[4], b=[2], c=[7]", &foo!(a = 4, c = 7));
    assert_eq!("a=[1], b=[9], c=[8]", &foo!(b = 9, c = 8));

    // 3 specified
    assert_eq!("a=[9], b=[8], c=[7]", &foo!(a = 9, b = 8, c = 7));
}
