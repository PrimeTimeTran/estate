fn main_a() {
	#[derive(Debug)]
	struct Node {
		val: i32,
		children: Vec<Node>,
	}
	let mut a = Node {
		val: 1,
		children: vec![],
	};
	let b = Node {
		val: 2,
		children: vec![],
	};
	a.children.push(b);
	dbg!(&a);
	// Errors due to b being moved into a's children
	// dbg!(&b);
}

fn main_a_solution() {
	#[derive(Debug)]
	struct Node<'a> {
		val: i32,
		children: Vec<&'a Node<'a>>,
	}
	let mut a = Node {
		val: 1,
		children: vec![],
	};
	let b = Node {
		val: 2,
		children: vec![],
	};
	a.children.push(&b);

	dbg!(&a);
	// No problem now!
	dbg!(&b);
}

fn main_b() {
	#[derive(Debug)]
	struct Node<'a> {
		val: i32,
		children: Vec<&'a Node<'a>>,
	}
	let mut a = Node {
		val: 1,
		children: vec![],
	};
	let b = Node {
		val: 2,
		children: vec![],
	};
	a.children.push(&b);
	for child in &a.children {
		dbg!(child);
		// cannot assign to `child.val`, which is behind a `&` reference
		// child.val += 1;
	}
}
fn main_b_solution() {
	use std::cell::Cell;
	#[derive(Debug)]
	struct Node<'a> {
		val: Cell<i32>,
		children: Vec<&'a Node<'a>>,
	}

	let mut a = Node {
		val: Cell::new(1),
		children: vec![],
	};
	let b = Node {
		val: Cell::new(2),
		children: vec![],
	};

	a.children.push(&b);

	// Now you can iterate and modify even with an immutable reference
	for child in &a.children {
		// .get() and .set() allow mutation through shared references
		child.val.set(child.val.get() + 1);
	}

	println!("a: {:?}", a.val.get());
	println!("b: {:?}", b.val.get());
}

fn main() {
	main_b_solution();
}

// fn main2() {
//     use std::cell::Cell;
//     #[derive(Debug)]
//     struct Node<'a> {
//         val: Cell<i32>,
//         children: Vec<&'a Node<'a>>,
//     }
//     let mut a = Node { val: Cell::new(1), children: vec![] };
//     let b = Node { val: Cell::new(2), children: vec![] };
//     a.children.push(&b);
//     dbg!(&a);
//     a.val.set(a.val.get() + 1);
//     dbg!(&b);
//     b.val.set(b.val.get() + 1);
// }

// fn main2() {
//     #[derive(Debug)]
//     struct Node<'a> {
//         val: i32,
//         children: Vec<&'a Node<'a>>,
//     }
//     fn add_one(node: &mut Node) {
//         node.val += 1;
//     }
//     let mut a = Node { val: 1, children: vec![] };
//     let b = Node { val: 2, children: vec![] };
//     a.children.push(&b);
//     add_one(&mut a);
//     // This is the classic "aliasing XOR mutability" problem in Rust.
//     // The compiler is strictly enforcing the rule: You can have either many immutable references
//     // (readers) OR one mutable reference (writer), but never both at the same time.
//     // add_one(&mut b);
//     dbg!(&a);
//     dbg!(&b);
// }

// // fn main() {
// //     use std::rc::Rc;
// //     use std::cell::RefCell;
// //     #[derive(Debug)]
// //     struct Node {
// //         val: i32,
// //         children: Vec<Rc<RefCell<Node>>>,
// //     }
// //     fn add_one(node: &Rc<RefCell<Node>>) {
// //         node.borrow_mut().val += 1;
// //     }
// //     let a = Rc::new(RefCell::new(Node { val: 1, children: vec![] }));
// //     let b = Rc::new(RefCell::new(Node { val: 2, children: vec![] }));

// //     // Add b to a's children
// //     a.borrow_mut().children.push(b.clone());

// //     // Both can be mutated independently
// //     add_one(&a);
// //     add_one(&b);

// //     println!("a: {:?}", a.borrow());
// //     println!("b: {:?}", b.borrow());

// // }
