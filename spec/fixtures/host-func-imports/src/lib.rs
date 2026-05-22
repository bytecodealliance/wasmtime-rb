#[allow(warnings)]
mod bindings;

use bindings::{math, Color, Guest, Permissions, Point, Shape};

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

    fn test_get_numbers() -> Vec<i32> {
        bindings::get_numbers()
    }

    fn test_make_tuple(n: u32, s: String, b: bool) -> (u32, String, bool) {
        bindings::make_tuple(n, &s, b)
    }

    fn test_analyze_numbers(numbers: Vec<i32>) -> (i32, Vec<i32>) {
        bindings::analyze_numbers(&numbers)
    }

    fn test_s8(n: i8) -> i8 {
        bindings::echo_s8(n)
    }

    fn test_u8(n: u8) -> u8 {
        bindings::echo_u8(n)
    }

    fn test_s16(n: i16) -> i16 {
        bindings::echo_s16(n)
    }

    fn test_u16(n: u16) -> u16 {
        bindings::echo_u16(n)
    }

    fn test_s64(n: i64) -> i64 {
        bindings::echo_s64(n)
    }

    fn test_u64(n: u64) -> u64 {
        bindings::echo_u64(n)
    }

    fn test_f32(n: f32) -> f32 {
        bindings::echo_f32(n)
    }

    fn test_f64(n: f64) -> f64 {
        bindings::echo_f64(n)
    }

    fn test_char(c: char) -> char {
        bindings::echo_char(c)
    }

    fn test_enum(c: Color) -> Color {
        bindings::echo_enum(c)
    }

    fn test_variant(s: Shape) -> Shape {
        bindings::echo_variant(s)
    }

    fn test_flags(p: Permissions) -> Permissions {
        bindings::echo_flags(p)
    }
}

bindings::export!(Component with_types_in bindings);
