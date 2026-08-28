struct Point<T> {
	x: T,
	y: T,
}

impl<T> Point<T> {
	fn new(x: T, y: T) -> Self {
		Self { x, y }
	}
}

trait Mathable: Sized + Copy {
	fn sub(self, other: Self) -> Self;
	fn mul(self, other: Self) -> Self;
	fn add(self, other: Self) -> Self;
	fn sqrt(self) -> Self;
}

impl Mathable for f64 {
	fn sub(self, other: Self) -> Self {
		self - other
	}
	fn mul(self, other: Self) -> Self {
		self * other
	}
	fn add(self, other: Self) -> Self {
		self + other
	}
	fn sqrt(self) -> Self {
		self.sqrt()
	}
}

impl<T: Mathable> Point<T> {
	fn distance(&self, other: &Point<T>) -> T {
		let dx = self.x.sub(other.x);
		let dy = self.y.sub(other.y);
		dx.mul(dx).add(dy.mul(dy)).sqrt()
	}
}

pub fn three() {
	let point1 = Point::new(1.0, 2.0);
	let point2 = Point::new(4.0, 6.0);
	println!("Distance: {}", point1.distance(&point2));
}
