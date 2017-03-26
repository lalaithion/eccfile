use bitvec::BitVec;

fn is_power_of_two(n: usize) -> bool {
    (n & (n - 1)) == 0
}

fn append(myself: BitVec, other: BitVec) -> BitVec {
    let mut result = BitVec::from_elem(myself.len() + other.len(), false);
    for i in 0..myself.len() {
        result.set(i, myself[i]);
    }
    for i in 0..other.len() {
        result.set(myself.len() + i, other[i]);
    }
    result
}

fn check(mut block: BitVec, p: usize) -> BitVec {
    let mut errors: Vec<usize> = Vec::new();
    // for each parity bit
    for n in 0..p {
        // the index is 2 ^ n
        let pindex = 1 << n;
        let mut parity = false;
        // we don't need to look at any indicies lower than pindex
        for i in (pindex)..block.len() {
            // if this index has the nth bit set
            if pindex & (i+1) != 0 {
                // xor its value with the parity value
                parity = block[i] ^ parity;
            }
        }
        // if the computed parity doesn't match the recorded parity, put it as an error
        if parity != block[pindex-1] {
            errors.push(pindex);
        }
    }
        
    // if there's one error, then the parity bit was wrong
    if errors.len() == 1 {
        let fix = ! block[errors[0]-1];
        block.set(errors[0]-1, fix);
    } else if errors.len() > 1 {
    // otherwise, add the parity indexes to get the value where they overlap, and flip it
        let mut errindex = 0;
        for i in errors {
            errindex += i;
        }
        let fix = ! block[errindex-1];
        block.set(errindex-1, fix);
    }
    
    block
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


fn parity(mut block: BitVec, p: usize) -> BitVec {
    // for each parity bit
    for n in 0..p {
        // calculate 2^n
        let pindex = 1 << n;
        let mut parity = false;
        // we don't need to look at any indicies lower than pindex
        for i in (pindex)..block.len() {
            // if this index has the nth bit set
            if pindex & (i+1) != 0 {
                // xor its value with the parity value
                parity = block[i] ^ parity;
            }
        }
        // set the 2^nth bit to the parity
        block.set(pindex-1, parity);
    }
    block
}

fn arrange(plain: &BitVec, start: usize, p: usize) -> (usize, BitVec) {
    let mut block = BitVec::with_capacity((1 << p)-1);
    let mut index = start;
    // for each item in the block
    for i in 0..((1 << p)-1) {
        // if it's a power of two, reserve the bit for parity
        // and if we're at the end of the plaintext, just push 0.
        if is_power_of_two(i+1) || index > plain.len() {
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
        let (new_index, temp_block) = arrange(&plain, index, p);
        let block = parity(temp_block, p);
        code = append(code, block);
        index = new_index;
    }
    code.to_bytes()
}

pub fn decode(v: &Vec<u8>, p: usize) -> Vec<u8> {
    let code = BitVec::from_bytes(&v);
    let mut plain = BitVec::with_capacity(code.len() / 2);
    let mut index = 0;
    let length = (1 << p) - 1;
    while index < code.len() {
        if index + length >= code.len() {
            break;
        }
        let mut block = BitVec::with_capacity(length);
        for i in index..(index+length) {
            block.push(code[i]);
        }
        block = check(block, p);
        block = assemble(block, p);
        plain = append(plain, block);
        index = index+length;
    }
    plain.to_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn power_of_two_works() {
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
    
    #[test]
    fn append_works() {
        let test: [u8;3] = [4,19,23];
        let other: [u8;3] = [2,11,48];
        let total: [u8;6] = [4,19,23,2,11,48];
        
        assert_eq!(BitVec::from_bytes(&total), append(BitVec::from_bytes(&test),BitVec::from_bytes(&other)))
    }
    
    #[test]
    fn check_null() {
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
        
        assert_eq!(check(corrupt1, 3), perfect1);
    }

    #[test]
    fn check_full() {
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
        
        assert_eq!(check(corrupt, 3), perfect);
    }

    #[test]
    fn check_perfect() {
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
        
        assert_eq!(check(corrupt, 3), perfect);
    }
    
    #[test]
    fn check_errors() {
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
        corrupt.push(false); // d 3
        corrupt.push(false); // p 4
        corrupt.push(false); // d 5
        corrupt.push(false); // d 6
        corrupt.push(false); // d 7
        
        assert_eq!(check(corrupt, 3), perfect);
        
        perfect = BitVec::new();
        perfect.push(true); // p 1
        perfect.push(false); // p 2
        perfect.push(false); // d 3
        perfect.push(true); // p 4
        perfect.push(true); // d 5
        perfect.push(false); // d 6
        perfect.push(false); // d 7

        corrupt = BitVec::new();
        corrupt.push(true); // p 1
        corrupt.push(false); // p 2
        corrupt.push(false); // d 3
        corrupt.push(true); // p 4
        corrupt.push(false); // d 5
        corrupt.push(false); // d 6
        corrupt.push(false); // d 7
        
        assert_eq!(check(corrupt, 3), perfect);
    }
    
    #[test]
    fn assemble_simple() {
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
    
    #[test]
    fn parity_simple() {
        let mut after = BitVec::new();
        after.push(true); // p 1
        after.push(false); // p 2
        after.push(true); // d 3
        after.push(true); // p 4
        after.push(false); // d 5
        after.push(true); // d 6
        after.push(false); // d 7

        // all paritys are false
        let mut before = BitVec::new();
        before.push(false); // p 1
        before.push(false); // p 2
        before.push(true); // d 3
        before.push(false); // p 4
        before.push(false); // d 5
        before.push(true); // d 6
        before.push(false); // d 7
        
        assert_eq!(parity(before, 3), after);
        
        after = BitVec::new();
        after.push(true); // p 1
        after.push(false); // p 2
        after.push(false); // d 3
        after.push(true); // p 4
        after.push(true); // d 5
        after.push(false); // d 6
        after.push(false); // d 7

        before = BitVec::new();
        before.push(false); // p 1
        before.push(false); // p 2
        before.push(false); // d 3
        before.push(false); // p 4
        before.push(true); // d 5
        before.push(false); // d 6
        before.push(false); // d 7
            
        assert_eq!(parity(before, 3), after);
    }
    
    #[test]
    fn arrange_simple() {
        let v = vec![18,42,5];
        let plain = BitVec::from_bytes(&v);
        
        let mut arranged = BitVec::new();
        arranged.push(false); // p 1
        arranged.push(false); // p 2
        arranged.push(false); // d 3
        arranged.push(false); // p 4
        arranged.push(false); // d 5
        arranged.push(false); // d 6
        arranged.push(true); // d 7

        let (i, result) = arrange(&plain, 0, 3);

        assert_eq!(result, arranged);
        
        arranged = BitVec::new();
        arranged.push(false); // p 1
        arranged.push(false); // p 2
        arranged.push(false); // d 3
        arranged.push(false); // p 4
        arranged.push(false); // d 5
        arranged.push(true); // d 6
        arranged.push(false); // d 7
        
        let (i, result) = arrange(&plain, i, 3);
        
        assert_eq!(result, arranged);
    }
    
    #[test]
    fn inverse() {
        let test1: Vec<u8> = vec![1,1,2,3,5,8,13,21,34];
        let test2: Vec<u8> = vec![11,7,25];
        let test3: Vec<u8> = vec![255,255,0,255,255,0];
        let test4: Vec<u8> = vec![0,0,0];
        let test5: Vec<u8> = vec![128,32,2];
        
        assert_eq!(decode(&encode(&test1, 3), 3), test1);
        assert_eq!(decode(&encode(&test2, 3), 3), test2);
        assert_eq!(decode(&encode(&test3, 3), 3), test3);
        assert_eq!(decode(&encode(&test4, 3), 3), test4);
        assert_eq!(decode(&encode(&test5, 3), 3), test5);
    }
}
