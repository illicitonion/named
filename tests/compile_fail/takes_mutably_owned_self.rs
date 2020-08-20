use named::named;

struct S {}

impl S {
    #[named]
    fn always_true(mut self) -> bool {
        true
    }
}

fn main() {}
