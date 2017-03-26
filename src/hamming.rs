use bitvec::BitVec;

fn is_power_of_two(n: usize) -> bool {
    (n & (n - 1)) == 0
}

#[test]
fn test_power_of_two() {
    // true
    assert!(is_power_of_two(2));
    assert!(is_power_of_two(4));
    assert!(is_power_of_two(8));
    assert!(is_power_of_two(1024));
    assert!(is_power_of_two(4096));
    
    // false
    assert!(!is_power_of_two(3));
    assert!(!is_power_of_two(5));
    assert!(!is_power_of_two(7));
    assert!(!is_power_of_two(10));
    assert!(!is_power_of_two(6));
}

fn check(mut block: BitVec, p: usize) -> BitVec {
    let mut errors: Vec<usize> = Vec::new();
    // for each parity bit
    for n in 0..p {
        // the index is 2 ^ n
        let pindex = 1 << n;
        let mut parity = false;
        // we don't need to look at any indicies lower than pindex + 1
        for i in (pindex)..block.len() {
            // if this index has the nth bit set
            if pindex & (i+1) != 0 {
                // xor its value with the parity value
                parity = block[i] ^ parity;
            }
        }
        // if the computed parity doesn't match the recorded parity, put it as an error
        if parity != block[pindex] {
            errors.push(pindex);
        }
    }
        
    // if there's one error, then the parity bit was wrong
    if errors.len() == 1 {
        let fix = ! block[errors[0]];
        block.set(errors[0], fix);
    } else if errors.len() > 1 {
    // otherwise, add the parity indexes to get the value where they overlap, and flip it
        let mut errindex = 0;
        for i in errors {
            errindex += i;
        }
        let fix = ! block[errindex];
        block.set(errindex, fix);
    }
    
    block
}

#[test]
fn test_check_null() {
    let mut perfect1 = BitVec::new();
    perfect1.push(false); // p 1
    perfect1.push(false); // p 2
    perfect1.push(false); // d 3
    perfect1.push(false); // p 4
    perfect1.push(false); // d 5
    perfect1.push(false); // d 6
    perfect1.push(false); // d 7

    let mut corrupt1 = BitVec::new();
    corrupt1.push(false); // p 1
    corrupt1.push(false); // p 2
    corrupt1.push(false); // d 3
    corrupt1.push(false); // p 4
    corrupt1.push(false); // d 5
    corrupt1.push(false); // d 6
    corrupt1.push(false); // d 7
    
    println!("null");
    assert_eq!(check(corrupt1, 3), perfect1);
}

#[test]
fn test_check_full() {
    let mut perfect = BitVec::new();
    perfect.push(true); // p 1
    perfect.push(true); // p 2
    perfect.push(true); // d 3
    perfect.push(true); // p 4
    perfect.push(true); // d 5
    perfect.push(true); // d 6
    perfect.push(true); // d 7

    let mut corrupt = BitVec::new();
    corrupt.push(true); // p 1
    corrupt.push(true); // p 2
    corrupt.push(true); // d 3
    corrupt.push(true); // p 4
    corrupt.push(true); // d 5
    corrupt.push(true); // d 6
    corrupt.push(true); // d 7
    
    println!("full");
    assert_eq!(check(corrupt, 3), perfect);
}

#[test]
fn test_check() {
    let mut perfect = BitVec::new();
    perfect.push(true); // p 1
    perfect.push(true); // p 2
    perfect.push(true); // d 3
    perfect.push(false); // p 4
    perfect.push(false); // d 5
    perfect.push(false); // d 6
    perfect.push(false); // d 7

    let mut corrupt = BitVec::new();
    corrupt.push(true); // p 1
    corrupt.push(true); // p 2
    corrupt.push(true); // d 3
    corrupt.push(false); // p 4
    corrupt.push(false); // d 5
    corrupt.push(false); // d 6
    corrupt.push(false); // d 7
    
    println!("normal");
    assert_eq!(check(corrupt, 3), perfect);
}

#[test]
fn test_check_errors() {
    let mut perfect = BitVec::new();
    perfect.push(true); // p 1
    perfect.push(true); // p 2
    perfect.push(true); // d 3
    perfect.push(false); // p 4
    perfect.push(false); // d 5
    perfect.push(false); // d 6
    perfect.push(false); // d 7

    let mut corrupt = BitVec::new();
    corrupt.push(true); // p 1
    corrupt.push(true); // p 2
    corrupt.push(true); // d 3
    corrupt.push(false); // p 4
    corrupt.push(false); // d 5
    corrupt.push(false); // d 6
    corrupt.push(false); // d 7
    
    println!("normal");
    assert_eq!(check(corrupt, 3), perfect);
}

fn assemble(block: BitVec, p: usize) -> BitVec {
    let mut result = BitVec::with_capacity((1 << p) - p - 1);
    for i in 0..(1 << p) {
        // if it's a power of two, this is a parity bit
        if is_power_of_two(i+1) {
            ;
        } else {
            result.push(block[i]);
        }
    }
    result
}

#[test]
fn test_assemble() {
    let mut code = BitVec::new();
    code.push(false); // p 1
    code.push(true); // p 2
    code.push(false); // d 3
    code.push(false); // p 4
    code.push(true); // d 5
    code.push(false); // d 6
    code.push(true); // d 7

    let mut plain = BitVec::new();
    plain.push(false); // d 3
    plain.push(true); // d 5
    plain.push(false); // d 6
    plain.push(true); // d 7
    
    assert_eq!(assemble(code, 3), plain);
}



fn parity(mut block: BitVec, p: usize) -> BitVec {
    // for each parity bit
    for n in 0..p {
        // calculate 2^n
        let pindex = 1 << n;
        let mut parity = false;
        for i in 0..block.len() {
            // if this index has the nth bit set
            if pindex & (i+1) != 0 {
                // xor its value with the parity value
                parity = block[i] ^ parity;
            }
        }
        // set the 2^nth bit to the parity
        block.set(pindex, parity);
    }
    block
}

fn arrange(plain: &BitVec, start: usize, p: usize) -> (usize, BitVec) {
    let mut block = BitVec::with_capacity((1 << p)-1);
    let mut index = start;
    // for each item in the block
    for i in 1..((1 << p)+1) {
        // if it's a power of two, reserve the bit for parity
        // and if we're at the end of the plaintext, just push 0.
        if i & (i - 1) == 0 || index > plain.len() {
            block.push(false);
        } else {
            // otherwise, push the next plaintext value
            block.push( plain[index] );
            index += 1;
        }
    }
    (index,block)
}

pub fn encode(v: &Vec<u8>, p: usize) -> Vec<u8> {
    let plain = BitVec::from_bytes(&v);
    let mut code = BitVec::with_capacity(2 * plain.len());
    let mut index = 0;
    while index < plain.len() {
        let (i, b) = arrange(&plain, index, p);
        let mut block = parity(b, p);
        code.append(&mut block);
        index = i;
    }
    code.to_bytes()
}

pub fn decode(v: &Vec<u8>, p: usize) -> Vec<u8> {
    let code = BitVec::from_bytes(&v);
    let mut plain = BitVec::with_capacity(code.len() / 2);
    let mut index = 0;
    let length = (1 << p) - 1;
    while index < plain.len() {
        let mut block = BitVec::with_capacity(length);
        for i in index..(index+length) {
            block.push(code[i]);
        }
        block = check(block, p);
        block = assemble(block, p);
        plain.append(&mut block);
        index = index+length;
    }
    plain.to_bytes()
}
