extern crate ansi_term;
extern crate cargo;
extern crate serde;
extern crate serde_json;
extern crate which;

mod lib;

use lib::StrErr;

fn main() -> Result<(), StrErr> {
	lib::main()
}
