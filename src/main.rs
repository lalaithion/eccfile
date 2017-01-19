extern crate argparse;

use std::io::{self, Read, Write};
use argparse::{ArgumentParser, StoreFalse, Store};

mod multiple;
mod bitvec;

fn main() {
    let mut number = 3;
    let mut encode = true;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("");
        ap.refer(&mut encode)
            .add_option(&["-d", "--decode"], StoreFalse,
            "use this flag to decode a file as opposed to encoding it");
        ap.refer(&mut number)
            .add_option(&["-n","--number"], Store,
            "use this flag to specify the number of duplicate bits to use \
             - odd numbers are more efficient than even numbers");
        ap.parse_args_or_exit();
    }
    
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).expect("There was an error while reading from stdin.");
    
    let output = if encode {
        multiple::encode(&input, number)
    } else {
        multiple::decode(&input, number)
    };
    
    io::stdout().write_all(&output).expect("There was an error while writing to stdout");
}
