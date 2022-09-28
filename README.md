The `Builder` derive macro creates a compile-time correct builder
which means that it only allows you to build the given struct if and only if you provide a
value for all of its required fields.

From the perspective of the builder there are three types of fields:
* **Optional Fields** which are fields wrapped in an `Option`.
* **Default Fields** which are given a default value through the `#[builder(default)]` attribute.
* **Required Fields** which are fields that do not fall into the previous categories.

Example below depicts these three types of fields:
```rust
use tidy_builder::Builder;

#[derive(Builder)]
struct Person {
    first_name: String,
    last_name: String,

    age: Option<usize>,

    #[builder(default = false)]
    employed: bool,
}

fn main() {
    let person = Person::builder()
                        .first_name("Foo".to_string())
                        .last_name("Bar".to_string())
                        .age(18)
                        .build();

    assert_eq!(person.first_name, "Foo".to_string());
    assert_eq!(person.last_name, "Bar".to_string());
    assert_eq!(person.age, Some(18));
    assert_eq!(person.employed, false);
}
```
As you can see, `first_name` and `last_name` are required fields, `age` is optional, and `employed` takes a default value of `false`. 
As we mentioned, in order to call `build`, you have to at least provide values for `first_name` and `last_name`.

# Features
## Repeated setters
For fields that are of form `Vec<T>`, you can instruct the builder to create a repeated setter for you. 
This repeated setter gets a single value of type `T` and appends to the `Vec`. For example:
```rust
use tidy_builder::Builder;

#[derive(Builder)]
struct Input<'a> {
    #[builder(each = "arg")]
    args: Vec<&'a str>
}

fn main() {
    let input1 = Input::builder().arg("arg1").arg("arg2").build();
    let input2 = Input::builder().args(vec!["arg1", "arg2"]).build();

    assert_eq!(input1.args, vec!["arg1", "arg2"]);
    assert_eq!(input2.args, vec!["arg1", "arg2"]);
}
```
The builder will create another setter function named `arg` alongside the `args` function that was going to be generated anyway. 
**Note** that if the name provided for the repeated setter is the same name as the field itself, 
only the repeated setter will be provided by the builder since Rust does not support function overloading. 
For example if in the example above the repeated setter was named `args`, the setter that takes a `Vec` wouldn't be provided.

## Default values
You can provide default values for fields and make them non-required. If the field is a primitive or a `String`, 
you can specify the default value in the `#[builder(default)]` attribute, but if the field is not a primitive, it must implement the `Default` trait. For example:
```rust
use tidy_builder::Builder;

#[derive(Debug, PartialEq)]
pub struct Point {
    x: usize,
    y: usize,
}

impl Default for Point {
    fn default() -> Self {
        Point {
            x: 0,
            y: 0,
        }
    }
}

#[derive(Builder)]
struct PlayerPosition {
    #[builder(default)]
    start: Point,

    #[builder(default = 0)]
    offset: usize,
}

fn main() {
    let position = PlayerPosition::builder().build();

    assert_eq!(position.start, Point { x: 0, y: 0});
    assert_eq!(position.offset, 0);
}
```

## Skipping Fields
You can prevent the builder from providing setters for **optional** and **default** fields. For example:
```rust compile_fail
use tidy_builder::Builder;

#[derive(Builder)]
struct Vote {
    submit_url: String,

    #[builder(skip)]
    name: Option<String>,

    #[builder(skip)]
    #[builder(default = false)]
    vote: bool
}

fn main() {
    let vote = Vote::builder().submit_url("fake_submit_url.com").name("Foo".to_string()); // Fails since there is no `name` setter
}
```

# What if I try to call the `build` function early?
tidy-builder uses special traits to hint at the missing required fields. For example:
```rust compile_fail
use tidy_builder::Builder;

#[derive(Builder)]
struct Foo {
    bar: usize,
    baz: usize,
}

fn main() {
    let foo = Foo::builder().bar(0).build();
}
```
On stable Rust you'll get a **compile-time** error that the trait `HasBaz` is not implemented for the struct `FooBuilder<...>`. 
The trait `HasBaz` indicates that `FooBuilder` **has** a value for the **`baz`** field. 
So this trait not being implemented for `FooBuilder` means that a value is not specified for the `baz` field and that's why you cannot call the `build` function.

On nightly Rust and with the help of `rustc_on_unimplemented`, the `Builder` can hint at the compiler to 
show the message `missing baz` to inform the user that in order to call `build`, they should set the value of the `baz` field. 
**Note** that this is behind the `better_error` feature gate.

# How it works
tidy-builder creates a state machine in order to model the behavior of the builder. The generated builder has a const generic parameter of type `bool` 
for each required field to encode whether a value has been set for the field or not. For example:
```rust
use tidy_builder::Builder;

#[derive(Builder)]
struct Foo {
    bar: usize,
    baz: usize,
}
```
The struct above will cause this builder to get generated:
```rust
struct FooBuilder<const P0: bool, const P1: bool> {
    bar: Option<usize>,
    baz: Option<usize>,
}
```
The builder will start in the `FooBuilder<false, false>` state when you call the `builder` function of `Foo`:
```rust
let builder: FooBuilder<false, false> = Foo::builder();
let builder: FooBuilder<true, false> = Foo::builder().bar(0);
let builder: FooBuilder<true, true> = Foo::builder().bar(0).baz(1);

let foo = builder.build();

assert_eq!(foo.bar, 0);
assert_eq!(foo.baz, 1);
```
When you call the `bar` function  to set the value of the `bar` field, you cause the builder to transition to the `FooBuilder<true, false>` state:
Similarly, when you call the `baz` function, you cause the builder to transition to the `FooBuilder<false, true>` state. 
So when you set the value for both fields, you end up at the `FooBuilder<true, true>` state, 
and it's in this state that you can call the build function(the state that all const generic paramters are `true`):

The error reporting discussed in the previous section leverages these states to inform the user of the missing fields. 
For example `HasBar` trait will be implemented for `FooBuilder<true, P1>` , and `HasBaz` will be implemented for `FooBuilder<P0, true>`. 
The `build` function is guarded with a where clause to make sure the builder implements all these traits:
```rust
impl<const P0: bool, const P1: bool> FooBuilder<P0, P1> {
    fn build(self) -> Foo
    where
        Self: HasBar + HasBaz
    {
        // Safety:
        //
        // It's safe since HasBar and HasBaz are implemented
        // hence self.bar and self.baz both contain valid values.
        unsafe {
            Foo {
                bar: self.bar.unwrap_unchecked(),
                baz: self.baz.unwrap_unchecked(),
            }
        }
    }
}
```
So if you set the value of `bar` and not `baz`, since `HasBaz` won't be implemented for `FooBuilder<true, false>`, 
you'll get a compile-time error that calling `build` is not possible.
