use std::path::PathBuf;
use std::process::Output;
use std::fs::File;
use std::io::Write;

struct Command {
	path :PathBuf,
}

impl Command {
	fn new(loc :&str) -> Self {
		let out_dir = env!("OUT_DIR");
		let path = PathBuf::from(out_dir).join(loc);
		std::fs::create_dir_all(&path).unwrap();
		Command {
			path,
		}
	}
	fn cargo_toml(&self, content :&str) {
		self.file("Cargo.toml", content);
	}
	fn file(&self, file_name :&str, content :&str) {
		let path = self.path.join(file_name);
		println!("Creating file {:?}", path);
		let mut file = File::create(path).unwrap();
		write!(file, "{}", content).unwrap();
	}
	fn dir(&self, name :&str) {
		let path = self.path.join(name);
		std::fs::create_dir_all(path).unwrap();
	}
	fn output(&self) -> Output {
		std::process::Command::new("rustup")
			.args(&["run", "nightly"])
			.arg(target_dir().join("cargo-udeps"))
			.current_dir(&self.path)
			.output()
			.unwrap()
	}
}

// Taken from https://github.com/assert-rs/assert_cmd/blob/9379147429ff1eb8cb0766c696d1ae6141b66a33/src/cargo.rs#L201
fn target_dir() -> PathBuf {
	std::env::current_exe()
		.ok()
		.map(|mut path| {
			path.pop();
			if path.ends_with("deps") {
				path.pop();
			}
			path
		})
		.unwrap()
}

macro_rules! new_command {
	{} => {
		Command::new(&format!("{}_{}_{}", file!(), line!(), column!()))
	};
}

#[test]
fn unused_byteorder() {
	let cmd = new_command!();
	cmd.cargo_toml(r#"
		[workspace]
		[package]
		name = "test-a"
		version = "0.0.1"
		[dependencies]
		byteorder = "1.0.0"
	"#);
	cmd.dir("src");
	cmd.file("src/lib.rs", "");
	let output = cmd.output();
	let stdout = std::str::from_utf8(&output.stdout).unwrap();
	let stderr = std::str::from_utf8(&output.stderr).unwrap();
	println!("stdout: {}", stdout);
	println!("stderr: {}", stderr);
	assert!(stdout.contains("byteorder"));
}
