use std::io::{self, Read, Write};
use std::ops::{self, BitAnd, BitOr};
use std::fmt::Debug;

struct Repeater<I>
    where I: Iterator
{
    number: u64,
    count: u64,
    current: Option<I::Item>,
    underlying: I
}

trait Repeat: Iterator {
    fn repeat(self, n: u64) -> Repeater<Self>
        where Self::Item: Clone,
              Self: Sized,
    {
        Repeater{number: n, count: 0, current: None, underlying: self}
    }
}

impl<I> Repeat for I where I: Iterator {}

impl<I> Iterator for Repeater<I>
    where I: Iterator,
          I::Item: Clone,
{
    type Item = I::Item;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.count == 0 {
            self.current = self.underlying.next();
        }
        self.count = (self.count + 1) % self.number;
        self.current.clone()
    }
}

#[test]
fn test_repeater() {
    let x = [1];
    let y = [2,3];
    for i in x.into_iter().repeat(5) {
        assert_eq!(*i, 1);
    }
    let yp : Vec<_> = y.into_iter().repeat(2).collect();
    assert_eq!(yp.len(), 4);
    assert_eq!(*yp[0], 2);
    assert_eq!(*yp[1], 2);
    assert_eq!(*yp[2], 3);
    assert_eq!(*yp[3], 3);
}

struct Collapser<I>
    where I: Iterator
{
    number: usize,
    underlying: I
}

trait Collapse: Iterator {
    fn collapse(self, n: usize) -> Collapser<Self>
        where Self::Item: Clone,
              Self: Sized,
    {
        Collapser{number: n, underlying: self}
    }
}

impl<I> Collapse for I where I: Iterator {}

impl<I, Item> Iterator for Collapser<I>
    where I: Iterator<Item=Item>,
          Item: BitAnd<Output=Item> + BitOr<Output=Item> + Clone + Debug,
{
    type Item = Item;
    
    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = Vec::with_capacity(self.number);
        for _ in 0..self.number {
            buffer.push(match self.underlying.next() {
                Some(b) => b,
                _ => return None
            });
        }
        let mut or = Vec::with_capacity(self.number);
        for i in 1..self.number {
            or.push(buffer[i].clone() | buffer[i - 1].clone());
        }
        let mut last = buffer[0].clone() | buffer[self.number - 1].clone();
        for i in 1..(self.number-1) {
            last = last & or[i].clone();
        }
        Some(last)
    }
}

#[test]
fn test_collapser() {
    let x: Vec<u8> = vec![11, 43, 9, 44, 192, 31];
    let xp: Vec<u8> = x.into_iter().collapse(3).collect();
    println!("{:?}",xp);
    assert_eq!(xp[0], 11 as u8);
    assert_eq!(xp[1], 12 as u8);
    assert_eq!(xp.len(), 2);
}

// TODO: Check this implementation, optimize, make more idiomatic
fn write_iterator<W: Write, I: Iterator<Item=u8>> (writer: &mut W, iter: I) {
    const SIZE: usize = 1024;

    let mut buffer = [0u8; SIZE];
    let mut index = 0;

    for i in iter {
        buffer[index] = i;

        index += 1;
        if index == SIZE {
            match writer.write_all(&buffer) {
                Ok(_) => 1,
                _ => panic!("There was an error writing from stdout")
            };
        }
    }

    match writer.write_all(&buffer[..index]) {
        Ok(_) => 1,
        _ => panic!("There was an error writing from stdout")
    };
}

fn main() {
    write_iterator(&mut io::stdout(),
        io::stdin().bytes().map(
            |x| match x {
                Ok(b) => b,
                _ => panic!("There was an error reading from stdin"),
            }
        ).repeat(3),
    );
}
