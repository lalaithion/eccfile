use bitvec::BitVec;

fn check(mut block: BitVec, p: usize) -> BitVec {
    let mut errors: Vec<usize> = Vec::new();
    let mut result = BitVec::with_capacity(block.len());
    // for each parity bit
    for n in 0..p {
        let pindex = 1 << n;
        let mut parity = false;
        for i in 0..block.len() {
            // if this index has the nth bit set
            if pindex & i != 0 {
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

fn assemble(block: BitVec, p: usize) -> BitVec {
    let mut result = BitVec::with_capacity((1 << p) - p - 1);
    for i in 0..(1 << p) {
        // if it's a power of two, this is a parity bit
        if i & (i - 1) == 0 {
            ;
        } else {
            result.push(block[i]);
        }
    }
    result
}

fn parity(mut block: BitVec, p: usize) -> BitVec {
    // for each parity bit
    for n in 0..p {
        // calculate 2^n
        let pindex = 1 << n;
        let mut parity = false;
        for i in 0..block.len() {
            // if this index has the nth bit set
            if pindex & i != 0 {
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
    for i in 0..(1 << p) {
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
