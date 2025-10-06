#[derive(strum::Display,)]
enum QwQ {
	A,
	B(String,),
	#[strum(to_string = "C has {x}")]
	C {
		x: i32,
	},
	#[strum(to_string = "technically, E")]
	D,
}

fn main() {
	let a = QwQ::A;
	let b = QwQ::B("wth".to_string(),);
	let c = QwQ::C { x: 666, };
	let d = QwQ::D;

	println!("a: {a}");
	println!("b: {b}");
	println!("c: {c}");
	println!("d: {d}");
}
