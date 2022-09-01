The `Builder` derive macro creates a compile-time correct builder.
It means that it only allows you to build the given struct as long as you provide a
value for all of its required fields.

A field is interpreted as required if it's not wrapped in an `Option`.
Any field inside of an `Option` is not considered required in order to
build the given struct. For example in:
```rust
pub struct MyStruct {
    foo: String,
    bar: Option<usize>,
}
```
The `foo` field is required and `bar` is optional. **Note** that although
`std::option::Option` also referes to the same type, for now this macro doesn't
recongnize anything other than `Option`.
The builder generated using the `Builder` macro guarantees correctness
by encoding the initialized set using const generics. An example makes it clear. Let's assume
we have a struct that has two required fields and an optional one:
```rust
pub struct MyStruct {
    req1: String,
    req2: String,
    opt1: Option<String>
}
```
The generated builder will be:
```rust
pub struct MyStructBuilder<const P0: bool, const P1: bool> {
    req1: Option<String>,
    req2: Option<String>,
    opt1: Option<String>,
}
```
The `P0` indicates whether the first required parameter is initialized or not. And similarly,
the `P1` does the same thing for the second required parameter. The initial state of the
builder will be `MyStructBuilder<false, false>` and the first time a required field is
initialized, its corresponding const generic parameter will be set to true which indicates a
different state. Setting an optional value does not change the state and consequently keeps the
same const generic parameters. When the builder reaches the `MyStructBuilder<true, true>` and
only then you can call the `build` function on the builder.

So the complete generated code for the given example struct is:
```rust
pub struct MyStruct {
    req1: String,
    req2: String,
    opt1: Option<String>
}

pub struct MyStructBuilder<const P0: bool, const P1: bool> {
    req1: Option<String>,
    req2: Option<String>,
    opt1: Option<String>,
}

impl MyStruct {
    pub fn builder() -> MyStructBuilder<false, false> {
        MyStructBuilder {
            req1: None,
            req2: None,
            opt1: None,
        }
    }
}

impl<const P0: bool, const P1: bool> MyStructBuilder<P0, P1> {
    pub fn req1(self, req1: String) -> MyStructBuilder<true, P1> {
        MyStructBuilder {
            req1: Some(req1),
            req2: self.req2,
            opt1: self.opt1,
        }
    }
    pub fn req2(self, req2: String) -> MyStructBuilder<P0, true> {
        MyStructBuilder {
            req1: self.req1,
            req2: Some(req2),
            opt1: self.opt1,
        }
    }
    pub fn opt1(self, opt1: String) -> MyStructBuilder<P0, P1> {
        MyStructBuilder {
            req1: self.req1,
            req2: self.req2,
            opt1: Some(opt1),
        }
    }
}

impl MyStructBuilder<true, true> {
    pub fn build(self) -> MyStruct {
        unsafe {
            MyStruct {
                req1: self.req1.unwrap_unchecked(),
                req2: self.req2.unwrap_unchecked(),
                opt1: self.opt1,
            }
        }
    }
}
```


### Nightly features

#### Better error messages

In case of a missing fields, error message are like this:

```
   |
19 |         .build();
   |          ^^^^^ the trait `HasReq1` is not implemented for ...
   |
```

Which means `req1` is missing. If you are using a **nightly** rust compiler, you can
have better error messages:

```
error[E0277]: missing `field2`
  --> src/main.rs:13:10
   |
13 |         .build();
   |          ^^^^^ provide `field2` before calling `.build()`
   |
```

To enable this feature add `better_error` to the crate feature list:

```toml
tidy-builder = { version="*", features=["better_error"] }
```

then add `#![feature(rustc_attrs)]` at the top of your root level crate.