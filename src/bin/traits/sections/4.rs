pub fn dispatch() {
	let point1 = Point::new(1.0, 2.0);
	let point2 = Point::new(4.0, 6.0);
	println!("Distance: {}", point1.distance(&point2));
}


struct Point<T> {
	x: T,
	y: T,
}

impl<T> Point<T> {
	fn new(x: T, y: T) -> Self {
		Self { x, y }
	}
}

trait Numeric {
	type Output; // An associated type
	fn add(self, other: Self) -> Self::Output;
}

// You can now implement addition that returns a specific result type
impl Numeric for f32 {
	type Output = f64; // f32 + f32 can result in f64
	fn add(self, other: Self) -> Self::Output {
		(self as f64) + (other as f64)
	}
}

trait Distance {
	type Scalar;
	// When you define trait Distance, you aren't just saying "this object has a function."
	// Also "this object has a function that returns some kind of number,
	// and I’m going to call that number a Scalar."
	fn distance(&self, other: &Self) -> Self::Scalar;
}

// 1. Cleaning Up Complex Signatures
// - Instead of putting all constraints inside the < > brackets, you can break them down after the return type.
// 2. Bounding Associated Types
// - You can constrain types produced by other traits, like forcing an Iterator's item type to implement a specific trait.
// 3. Conditional Implementations (impl Blocks)
// - You can use where to conditionally implement a trait for a struct only if its underlying data type fulfills certain criteria.
impl<T> Distance for Point<T>
where
	T: Into<f64> + Copy,
{
	type Scalar = f64;

	fn distance(&self, other: &Point<T>) -> Self::Scalar {
		let dx = self.x.into() - other.x.into();
		let dy = self.y.into() - other.y.into();
		(dx * dx + dy * dy).sqrt()
	}
}

