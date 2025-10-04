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
                    let inner: usize = NumType::to_int();
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
                    let inner: i32 = NumType::to_int();
                    assert_eq!(inner, i);
                    assert!(inner < 0);
                },
                P => {
                    let inner: i32 = NumType::to_int();
                    assert_eq!(inner, i);
                    assert!(inner > 0);
                },
                False => {
                    let inner: u8 = NumType::to_u8();
                    assert_eq!(inner, i as u8);
                    assert!(inner == 0);
                }
            }
        }
    }
}

#[test]
fn literal_match_test() {
    // Test positive literal
    let result_positive = u_num_it! {
        -10..=10,
        match 5 {
            5 => "matched positive 5",
            N => "negative",
            P => "other positive",
            False => "zero",
            _ => "fallback"
        }
    };
    assert_eq!(result_positive, "matched positive 5");

    // Test negative literal
    let result_negative = u_num_it! {
        -10..=10,
        match -3 {
            -3 => "matched negative 3",
            N => "other negative",
            P => "positive",
            False => "zero",
            _ => "fallback"
        }
    };
    assert_eq!(result_negative, "matched negative 3");

    // Test zero literal
    let result_zero = u_num_it! {
        -10..=10,
        match 0 {
            0 => "matched zero",
            N => "negative",
            P => "positive",
            _ => "fallback"
        }
    };
    assert_eq!(result_zero, "matched zero");
}

#[test]
fn num_type_test() {
    // Test that NumType is available in the match arms
    for i in -5..5 {
        u_num_it! {
            -5..5,
            match i {
                N => {
                    // NumType should be the resolved typenum type
                    let inner: i32 = NumType::to_int();
                    assert_eq!(inner, i);
                    assert!(inner < 0);
                },
                P => {
                    let inner: i32 = NumType::to_int();
                    assert_eq!(inner, i);
                    assert!(inner > 0);
                },
                False => {
                    let inner: u8 = NumType::to_u8();
                    assert_eq!(inner, i as u8);
                    assert!(inner == 0);
                }
            }
        }
    }
    
    // Test literal case with NumType
    let result = u_num_it! {
        -10..=10,
        match 5 {
            5 => {
                // NumType should be typenum::consts::P5
                let val: i32 = NumType::to_int();
                assert_eq!(val, 5);
                "matched literal with NumType"
            },
            N => "negative",
            P => "other positive",
            False => "zero",
            _ => "fallback"
        }
    };
    assert_eq!(result, "matched literal with NumType");
}

