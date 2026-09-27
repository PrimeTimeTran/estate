fn main() {
    foo();
    bar();
    let fooz = Foo::new();
    let barz = Bar;
}

fn foo() {}
fn bar() {}
struct Foo {
}

impl Foo {
    pub fn new() -> Self {
        Self {
        }
    }
}

struct Bar {}
