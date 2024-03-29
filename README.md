# u-num-it

A simple procedural macros for matching `typenum::consts` in a given range.

Helps you to write

```rust
let x:isize = 3;

u_num_it!(-5..5, match x {
    N => {
        N::new()
    },
    False => {
        False::new()
    },
    P => {
        P::new()
    }
})
```

instead of

```rust
let x = 3;
match x {
    -5 => {
        typenum::consts::N5::new();
    }
    -4 => {
        typenum::consts::N4::new();
    }
    -3 => {
        typenum::consts::N3::new();
    }
    -2 => {
        typenum::consts::N2::new();
    }
    -1 => {
        typenum::consts::N1::new();
    }
    0 => {
        typenum::consts::False::new();
    }
    1 => {
        typenum::consts::P1::new();
    }
    2 => {
        typenum::consts::P2::new();
    }
    3 => {
        typenum::consts::P3::new();
    }
    4 => {
        typenum::consts::P4::new();
    }
    i => {
        panic!()
    }
}
```
