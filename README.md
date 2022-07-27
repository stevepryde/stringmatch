[![Crates.io](https://img.shields.io/crates/v/stringmatch.svg?style=for-the-badge)](https://crates.io/crates/stringmatch)
[![docs.rs](https://img.shields.io/badge/docs.rs-stringmatch-blue?style=for-the-badge)](https://docs.rs/stringmatch)

Allow the use of regular expressions or strings wherever you need string comparison.

# Examples

## Using Generics / Monomorphization

This pattern is faster but generates more code, slightly larger binary.

If you don't have a preference, go with this option as the code is usually more convenient to write.

```rust
fn accept_needle<N>(needle: N) -> bool
where
    N: Needle,
{
    needle.is_match("Test")
}
```

And now all of the following will work:
```rust
accept_needle("Test");
accept_needle(String::from("Test"));
accept_needle(Regex::new("Test").unwrap());
accept_needle(Regex::new(r"^T.+t$").unwrap());
```

For string comparisons you can also use `StringMatch` which allows you to be more explicit about the comparison:
```rust
accept_needle(StringMatch::from("test").case_insensitive());
accept_needle(StringMatch::from("tes").partial());
```
    
By default `StringMatch` matches the whole string and is case sensitive (safety by default).

And finally there is the `StringMatchable` trait that is implemented for `String` and `&str`:
```rust
accept_needle("test".match_case_insensitive());
accept_needle("tes".match_partial());
```

## Using Dynamic Dispatch

This pattern is slightly slower but generates less code, slightly smaller binary.

 ```rust
fn accept_needle(needle: &dyn Needle) -> bool
{
    needle.is_match("Test")
}
```

And now all of the following will work:
```rust
accept_needle(&"Test");
accept_needle(&String::from("Test"));
accept_needle(&Regex::new("Test").unwrap());
accept_needle(&Regex::new(r"^T.+t$").unwrap());
accept_needle(&StringMatch::from("test").case_insensitive());
accept_needle(&StringMatch::from("tes").partial());
```

## LICENSE

This work is licensed under MIT.

`SPDX-License-Identifier: MIT`
