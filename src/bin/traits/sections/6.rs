pub fn composition() {}

struct Point<T> {
	x: T,
	y: T,
}

impl<T> Point<T> {
	fn new(x: T, y: T) -> Self {
		Self { x, y }
	}
}

// 4. Use the trait in your generic implementation
impl<T> Point<T>
where
	T: ToFloat,
{
	fn distance(&self, other: &Point<T>) -> f64 {
		let dx = self.x.to_f64() - other.x.to_f64();
		let dy = self.y.to_f64() - other.y.to_f64();
		(dx * dx + dy * dy).sqrt()
	}
}

pub fn six() {
	let p1 = Point::new("1.0", "2.0");
	let p2 = Point::new("4.0", "6.0");
	println!("Distance (str): {}", p1.distance(&p2));

	let p3 = Point::new(1, 2);
	let p4 = Point::new(4, 6);
	println!("Distance (int): {}", p3.distance(&p4));
}

// 1. Define a trait that handles the conversion to f64
trait ToFloat {
	fn to_f64(&self) -> f64;
}

// 2. Implement it for &str
impl ToFloat for &str {
	fn to_f64(&self) -> f64 {
		self.parse().unwrap_or(0.0)
	}
}

// 3. Implement it for integers (and you could do the same for f32)
impl ToFloat for i32 {
	fn to_f64(&self) -> f64 {
		*self as f64
	}
}

impl ToFloat for f64 {
	fn to_f64(&self) -> f64 {
		*self
	}
}
