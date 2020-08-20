use named::named;

struct S {}

impl S {
    #[named]
    fn always_true(&self) -> bool {
        true
    }
}

fn main() {}
