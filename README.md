# u-num-it

A simple procedural macro for matching `typenum::consts` in a given range or array.

It helps you write concise numeric typenum matches like:

```rust
use typenum::{Bit, ToInt, Unsigned};

let x: isize = 3;

let y: isize = u_num_it::u_num_it!(-5..5, match x {
    N => {
        // negative integer types implement ToInt
        NumType::to_int()
    },
    False => {
        // False (B0) is a Bit, not an Integer. Use to_u8() and cast.
        NumType::to_u8() as isize
    },
    P => {
        // positive integer types implement ToInt
        NumType::to_int()
    }
});

assert_eq!(y, 3);
```

Or with an array of arbitrary numbers:

```rust
use typenum::{Bit, ToInt, Unsigned};

let x: isize = 8;

let y: isize = u_num_it::u_num_it!([1, 2, 8, 22], match x {
    P => {
        NumType::to_int()
    },
    _ => panic!("unexpected")
});

assert_eq!(y, 8);
```

Instead of writing the explicit (and repetitive) match yourself:

```rust
use typenum::{Bit, ToInt, Unsigned};

let x: isize = 3;

let y: isize = match x {
    -5 => typenum::consts::N5::to_int(),
    -4 => typenum::consts::N4::to_int(),
    -3 => typenum::consts::N3::to_int(),
    -2 => typenum::consts::N2::to_int(),
    -1 => typenum::consts::N1::to_int(),
     0 => typenum::consts::False::to_u8() as isize, // False is a Bit (B0)
     1 => typenum::consts::P1::to_int(),
     2 => typenum::consts::P2::to_int(),
     3 => typenum::consts::P3::to_int(),
     4 => typenum::consts::P4::to_int(),
    i  => panic!("out of range: {i}")
};

assert_eq!(y, 3);
```

`NumType` is available inside each match arm, bound to the concrete typenum type for the matched value:

```rust
use typenum::{Bit, ToInt};

let x: isize = 3;

let y: isize = u_num_it::u_num_it!(-5..5, match x {
    N => {
        // NumType is a negative integer type (e.g. typenum::consts::N3 when x = -3)
        NumType::to_int()
    },
    False => {
        // NumType is typenum::consts::False (B0). Use to_u8() and cast if you need an isize.
        let zero: u8 = NumType::to_u8();
        zero as isize
    },
    P => {
        // NumType is typenum::consts::P3 when x = 3
        let val: isize = NumType::to_int();
        val
    }
});

assert_eq!(y, 3);
```

## Notes

- Use `ToInt` for signed/unsigned integer typenum constants (`P*`, `N*`).
- Use `to_u8()` for bit types (`False` = `B0`, `True` = `B1` if you ever introduce it), casting as needed.
- Avoid mixing `P` and `U` in the same macro call (the macro enforces this).
- Literal `0` and `False` are treated as the same value; don't use both in one invocation.
