# named

Named arguments and default argument values for rust functions.

[![crates.io](https://img.shields.io/crates/v/named.svg)](https://crates.io/crates/named)
[![Documentation](https://docs.rs/named/badge.svg)](https://docs.rs/named)
[![Build Status](https://travis-ci.org/illicitonion/named.svg?branch=master)](https://travis-ci.org/illicitonion/named)

> ⚠️ **Warning:** This crate is intended as an experiment to explore potential ways to provide named arguments in Rust - while it _should_ work, I wouldn't necessarily encourage its use. In particular, it has significant limitations (such as not supporting functions inside `impl` blocks), and no real intention to work around the current language restrictions in order to remove them.

This procedural macro allows you to produce functions which can be called with named arguments, optionally with default values. The function must be called as a macro, rather than like a "real" function.
```rust
use named::named;

#[named(defaults(a = false, b = false))]
fn or(a: bool, b: bool) -> bool {
    a || b
}

fn main() {
    // You can use defaults for everything:
    assert!(!or!());

    // Or just for some values:
    assert!(or!(a = true));
    assert!(or!(b = true));
    assert!(!or!(a = false));
    assert!(!or!(b = false));

    // Or explicitly specify them all:
    assert!(or!(a = true, b = false));
    assert!(or!(a = false, b = true));
    assert!(or!(a = true, b = true));
    assert!(!or!(a = false, b = false));
}
```

Arguments must be specified in the same order as they were declared in the function, so this code is not ok:
```rust
use named::named;

#[named(defaults(a = false, b = false))]
fn or(a: bool, b: bool) -> bool {
    a || b
}

fn main() {
    assert!(or!(b = false, a = true));
}
```

All arguments must be supplied with names, you can't mix and match, so this code is not ok:

```rust
use named::named;

#[named(defaults(a = false, b = false))]
fn or(a: bool, b: bool) -> bool {
    a || b
}

fn main() {
    assert!(or!(a = true, false));
}
```

Not all arguments need default values; you could do this:
```rust
use named::named;

#[named(defaults(b = false))]
fn or(a: bool, b: bool) -> bool {
    a || b
}

fn main() {
    assert!(or!(a = true));
    assert!(or!(a = true, b = true));
}
```

Any const expression can be used as a default value:
```rust
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
}
```

All of the smarts happen at compile time, so at runtime this macro results in plain function calls with no extra overhead.

Unfortunately, this can't currently be used for functions defined in `impl` blocks, e.g. those which take a `self` parameter. It's possible that [postfix macros](https://github.com/rust-lang/rfcs/pull/2442) could enable this nicely.
