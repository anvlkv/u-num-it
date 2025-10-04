#[macro_use]
extern crate u_num_it;

use typenum::{Bit, ToInt};

fn main() {
    println!("NumType Demo - Testing with different values:\n");
    
    for x in -5..5 {
        let result = u_num_it! {
            -5..5,
            match x {
                N => {
                    // NumType is the resolved typenum type (e.g., N3 for -3)
                    let val: i32 = NumType::to_int();
                    format!("Negative value: {}", val)
                },
                P => {
                    // NumType is the resolved typenum type (e.g., P3 for 3)
                    let val: i32 = NumType::to_int();
                    format!("Positive value: {}", val)
                },
                False => {
                    // NumType is typenum::consts::False
                    let val: u8 = NumType::to_u8();
                    format!("Zero value: {}", val)
                }
            }
        };
        
        println!("x = {:<3} -> {}", x, result);
    }
}
