The run macro is a shorthand for making a main function.

You use it by prefixing your `Game` implementing `struct` with `#[rhachis::run]`. The following code sample:

```rust
use rhachis::*;
use rhachis::graphics::EmptyRenderer;

#[rhachis::run]
struct Run(EmptyRenderer);

impl Game for Run {
    // ...
}
```

is evaluated to:

```rust
use rhachis::*;
use rhachis::graphics::EmptyRenderer;

fn main() {
    Run::run()
}

struct Run(EmptyRenderer);

impl Game for Run {
    // ...
}
```

The main function is very often the same in all Rhachis projects which is why this shorthand is available, but you can still implement the main function yourself.
