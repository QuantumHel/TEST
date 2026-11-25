use circuit::Circuit;
use openqasm2::OpenQasm2Frontend;

pub struct OnlineOpenQasm2Source<T: 'static> {
	frontend: OpenQasm2Frontend<Circuit<T>>,
	links: Vec<(String, String)>,
}

impl<T: 'static> OnlineOpenQasm2Source<T> {
	pub fn new(frontend: OpenQasm2Frontend<Circuit<T>>, mut links: Vec<(String, String)>) -> Self {
		links.reverse();
		Self { frontend, links }
	}
}

impl<T: 'static> Iterator for OnlineOpenQasm2Source<T> {
	type Item = (String, Circuit<T>);
	fn next(&mut self) -> Option<Self::Item> {
		let (name, link) = self.links.pop()?;
		let body: String = ureq::get(link)
			.call()
			.unwrap()
			.body_mut()
			.read_to_string()
			.unwrap();
		let mut circuit = Circuit::default();
		self.frontend.compile_str(&body, &mut circuit).unwrap();
		Some((name, circuit))
	}
}
