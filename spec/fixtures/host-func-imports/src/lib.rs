#[allow(warnings)]
mod bindings;

use bindings::{math, Guest, Point};

struct Component;

impl Guest for Component {
    fn test_greet(name: String) -> String {
        bindings::greet(&name)
    }

    fn test_add(a: u32, b: u32) -> u32 {
        bindings::add(a, b)
    }

    fn test_constant() -> u32 {
        bindings::get_constant()
    }

    fn test_point(x: i32, y: i32) -> Point {
        bindings::make_point(x, y)
    }

    fn test_sum(numbers: Vec<i32>) -> i32 {
        bindings::sum_list(&numbers)
    }

    fn test_maybe(n: Option<u32>) -> Option<u32> {
        bindings::maybe_double(n)
    }

    fn test_divide(a: u32, b: u32) -> Result<u32, String> {
        bindings::safe_divide(a, b)
    }

    fn test_multiply(a: u32, b: u32) -> u32 {
        math::multiply(a, b)
    }
}

bindings::export!(Component with_types_in bindings);
