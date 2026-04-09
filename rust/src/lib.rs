use napi_derive_ohos::napi;

/// Add two unsigned integers.
#[napi]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

/// Compute the nth Fibonacci number (iterative).
#[napi]
pub fn fibonacci(n: u32) -> u32 {
    let (mut a, mut b) = (0u32, 1u32);
    for _ in 0..n {
        let tmp = b;
        b = a.wrapping_add(b);
        a = tmp;
    }
    a
}

/// Reverse a string.
#[napi]
pub fn reverse_string(s: String) -> String {
    s.chars().rev().collect()
}

/// Return a greeting built in Rust.
#[napi]
pub fn greet(name: String) -> String {
    format!("Hello {name}, from Rust on HarmonyOS!")
}
