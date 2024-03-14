#[macro_use]
extern crate u_num_it;

use typenum::{Bit, ToInt};

#[test]
fn u_macro_test() {
    for i in 1..=10 {
        u_num_it! {
            1..=10,
            match i {
                U => {
                    let inner: usize = U::to_int();
                    assert_eq!(inner, i);
                }
            }
        }
    }
}

#[test]
fn i_macro_test() {
    for i in -5..5 {
        u_num_it! {
            -5..5,
            match i {
                N => {
                    let inner: i32 = N::to_int();
                    assert_eq!(inner, i);
                    assert!(inner < 0);
                },
                P => {
                    let inner: i32 = P::to_int();
                    assert_eq!(inner, i);
                    assert!(inner > 0);
                },
                False => {
                    let inner: u8 = False::to_u8();
                    assert_eq!(inner, i as u8);
                    assert!(inner == 0);
                }
            }
        }
    }
}
