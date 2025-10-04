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

## NumType

As of version 0.2.1, each match arm includes a `NumType` type alias that resolves to the specific typenum type for that value. This allows you to reference the resolved type directly:

```rust
let x = 3;

u_num_it!(-5..5, match x {
    N => {
        // NumType is typenum::consts::N3 when x is -3
        let val: i32 = NumType::to_int();
        println!("Negative: {}", val);
    },
    P => {
        // NumType is typenum::consts::P3 when x is 3
        let val: i32 = NumType::to_int();
        println!("Positive: {}", val);
    },
    False => {
        // NumType is typenum::consts::False when x is 0
        let val: u8 = NumType::to_u8();
        println!("Zero: {}", val);
    }
})
```
