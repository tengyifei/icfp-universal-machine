mod um;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    assert!(args.len() == 2);
    let filename = &args[1];
    let program = fs::read(filename).expect("Unable to load program");
    let m = um::machine::Machine::new(program);
    m.execute().unwrap();
}
