- Cell
- RwLock
- RefCell
- Mutex

That is a wise decision. Trying to debug lifetime errors while simultaneously trying to architect a filesystem is like trying to learn how to fix a car engine while driving down the highway at 70 mph.

Since you are focusing on the fundamentals, let's look at the "Lifetime Paradox" that explains why your code was fighting you.

### The Lifetime Trap: What are you actually telling the compiler?

When you write `struct Node<'a> { children: Vec<&'a Node<'a>> }`, you are telling the compiler:
*"This `Node` contains references that **must** live at least as long as the `Node` itself."*

The compiler is enforcing this strictly because if it let you pass a reference to a `Node` that might "die" (go out of scope) while the parent `Node` was still trying to look at it, you would have a **dangling pointer**—one of the most dangerous bugs in systems programming.

### The 3 Rules of Thumb for Lifetimes

If you are reviewing the concepts, keep these three things in mind. They are the "Golden Rules" that govern why your code works (or doesn't):

1. **Lifetimes are about validity, not duration:** A lifetime doesn't track how long a variable lives; it tracks how long it is **valid to be read**.
2. **Shared vs. Exclusive:** You can have infinite readers (`&T`) as long as they are all gone before you have a single writer (`&mut T`).
3. **The "Shrinking" Rule:** A function can take a long-lived reference and treat it as a short-lived one, but never the other way around.