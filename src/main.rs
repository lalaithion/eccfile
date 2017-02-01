extern crate argparse;

use std::io::{self, Read, Write};
use argparse::{ArgumentParser, StoreFalse, Store, StoreTrue};

mod multiple;
mod hamming;
mod bitvec;

fn main() {
    let mut multiply = false;
    let mut hamm = false;
    let mut encode = true;
    let mut num = 3;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("");
        ap.refer(&mut encode)
            .add_option(&["-d", "--decode"], StoreFalse,
            "Use this flag to decode a file as opposed to encoding it.");
        ap.refer(&mut multiply)
            .add_option(&["-m","--multiply"], StoreTrue,
            "Use this option to indicate error correction method should be bit multiplication or duplication\
            where the numerical parameter used indicates the number of times each bit is duplicated. Odd numbers\
            are more efficient than even numbers");
        ap.refer(&mut hamm)
            .add_option(&["-h","--hamming"], StoreTrue,
            "Use this option to indicate error correction method should be hamming codes. The numerical parameter\
            indicates the number of parity bits; 3 hamming bits is equivalent to Hamming(7,4).");
        ap.refer(&mut num)
            .add_option(&["-n","--numerical"], Store,
            "Use this option to indicate the amount of repetition in the error correction method. For details, see\
            the different methods.");
        ap.parse_args_or_exit();
    }
    
    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input).expect("There was an error while reading from stdin.");
    
    let output: Vec<u8>;
    
    if multiply {
        if encode {
            output = multiple::encode(&input, num);
        } else {
            output = multiple::decode(&input, num);
        }
    } else {
        if encode {
            output = hamming::encode(&input, num);
        } else {
            output = hamming::decode(&input, num);
        }
    }
    
    io::stdout().write_all(&output).expect("There was an error while writing to stdout");
}
