# Pagination Macros

This crate provides procedural macros to simplify the implementation of pagination in Rust applications. It offers macros to automatically generate pagination logic for various data structures, making it easier to handle large datasets efficiently.

## Why Procedural Macros?

Procedural macros allow us to generate code at compile time, which can significantly reduce boilerplate and improve maintainability. By using procedural macros for pagination, we can automatically implement common pagination patterns without manually writing repetitive code.

## Why Use a Separate Crate?

You might wonder why this was not included in the [pagination](../pagination) crate.

From the [Rust Book](https://doc.rust-lang.org/book/ch20-05-macros.html#procedural-macros-for-generating-code-from-attributes):

> When creating procedural macros, the definitions must reside in their own crate with a special crate type. This is for complex technical reasons that we hope to eliminate in the future.

Macros need to be compiled in order to be usable at compile time. Therefore, we need to define them in a separate crate with the `proc-macro` crate type.

```toml
# Cargo.toml
[package]
name = "pagination-macros"

[lib]
proc-macro = true
```
