extern crate u_num_iter;

use u_num_iter::u_num_it;

struct Test<N> {
    field: N,
}

#[test]
fn macro_test() {
    for i in 1..=10 {
        u_num_it!(
            1..=10,
            match i {
                U => {
                    let val = Test::<U> { field: U::new() };
                    assert_eq!(val.field.to_int(), i)
                }
            }
        )
    }
}
