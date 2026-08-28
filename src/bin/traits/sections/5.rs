struct Point<T> {
	x: T,
	y: T,
}

impl<T> Point<T> {
	fn new(x: T, y: T) -> Self {
		Self { x, y }
	}
}

trait Parsable {
	fn parse_to_f64(&self) -> f64;
}

impl Parsable for &str {
	fn parse_to_f64(&self) -> f64 {
		self.parse().unwrap_or(0.0)
	}
}

impl<T> Point<T>
where
	T: Parsable,
{
	fn distance(&self, other: &Point<T>) -> f64 {
		let dx = self.x.parse_to_f64() - other.x.parse_to_f64();
		let dy = self.y.parse_to_f64() - other.y.parse_to_f64();
		(dx * dx + dy * dy).sqrt()
	}
}

pub fn five() {
	let point1 = Point::new("1.0", "2.0");
	let point2 = Point::new("4.0", "6.0");
	println!("Distance: {}", point1.distance(&point2));
}
