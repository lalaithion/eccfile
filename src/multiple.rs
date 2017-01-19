use bitvec::BitVec;

pub fn encode(v: &Vec<u8>, n: usize) -> Vec<u8> {
    let plain = BitVec::from_bytes(&v);
    let mut code = BitVec::with_capacity(plain.len() * n);
    
    for bit in plain.iter() {
        for _ in 0..n {
            code.push(bit);
        }
    }
    
    code.to_bytes()
}

pub fn decode(v: &Vec<u8>, n: usize) -> Vec<u8> {
    let code = BitVec::from_bytes(&v);
    let mut plain = BitVec::with_capacity(code.len() / n);
    let mut buffer = vec![0 as u8; n];
    let mut index = 0;
    
    for bit in code.iter() {
        buffer[index] = bit as u8;
        index += 1;
        
        if index == n {
            index = 0;
            let sum: u8 = buffer.iter().sum();
            let bit = if sum > (n as u8)/2 { 1 } else { 0 };
            plain.push(bit == 1);
        }
    }
    
    plain.to_bytes()
}

#[test]
fn inverse_test() {
    let test1: Vec<u8> = vec![1,1,2,3,5,8,13,21,34];
    let test2: Vec<u8> = vec![11,7,25,1];
    let test3: Vec<u8> = vec![255,255,0,255,255];
    let test4: Vec<u8> = vec![0,0,0];
    let test5: Vec<u8> = vec![128,32,2,4];
    
    assert_eq!(decode(&encode(&test1, 3), 3), test1);
    assert_eq!(decode(&encode(&test2, 17), 17), test2);
    assert_eq!(decode(&encode(&test3, 9), 9), test3);
    assert_eq!(decode(&encode(&test4, 3), 3), test4);
    assert_eq!(decode(&encode(&test5, 5), 5), test5);
}

#[test]
fn encode_test() {
    let test: Vec<u8> = vec![15];
    let other: Vec<u8> = vec![128];
    
    assert_eq!(encode(&test, 2), vec![0,255]);
    assert_eq!(encode(&test, 4), vec![0,0,255,255]);
    assert_eq!(encode(&test, 8), vec![0,0,0,0,255,255,255,255]);
    assert_eq!(encode(&other, 8), vec![255,0,0,0,0,0,0,0]);
}
