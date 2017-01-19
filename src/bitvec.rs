//Stolen from rust source; will use the standard version as soon as it is stable.

use std::cmp;
use std::fmt;
use std::hash;
#[allow(deprecated)]
use std::iter::{Chain, Enumerate, Repeat, Skip, Take, repeat, Cloned, FromIterator};
use std::mem::swap;
use std::ops::Index;
use std::slice;
use std::{u8, u32, usize};

const TRUE: &'static bool = &true;
const FALSE: &'static bool = &false;

type Blocks<'a> = Cloned<slice::Iter<'a, u32>>;
type MutBlocks<'a> = slice::IterMut<'a, u32>;
type MatchWords<'a> = Chain<Enumerate<Blocks<'a>>, Skip<Take<Enumerate<Repeat<u32>>>>>;

fn reverse_bits(byte: u8) -> u8 {
    let mut result = 0;
    for i in 0..8 {
        result |= ((byte >> i) & 1) << (8 - 1 - i);
    }
    result
}

pub struct BitVec {
    /// Internal representation of the bit vector
    storage: Vec<u32>,
    /// The number of valid bits in the internal representation
    nbits: usize
}

// FIXME(Gankro): NopeNopeNopeNopeNope (wait for IndexGet to be a thing)
impl Index<usize> for BitVec {
    type Output = bool;

    #[inline]
    fn index(&self, i: usize) -> &bool {
        if self.get(i).expect("index out of bounds") {
            TRUE
        } else {
            FALSE
        }
    }
}

/// Computes how many blocks are needed to store that many bits
fn blocks_for_bits(bits: usize) -> usize {
    // If we want 17 bits, dividing by 32 will produce 0. So we add 1 to make
    // sure we reserve enough. But if we want exactly a multiple of 32, this
    // will actually allocate one too many. So we need to check if that's the
    // case. We can do that by computing if bitwise AND by `32 - 1` is 0. But
    // LLVM should be able to optimize the semantically superior modulo operator
    // on a power of two to this.
    //
    // Note that we can technically avoid this branch with the expression
    // `(nbits + 32 - 1) / 32::BITS`, but if nbits is almost usize::MAX
    // this will overflow.
    if bits % 32 == 0 {
        bits / 32
    } else {
        bits / 32 + 1
    }
}

/// Computes the bitmask for the final word of the vector
fn mask_for_bits(bits: usize) -> u32 {
    // Note especially that a perfect multiple of 32 should mask all 1s.
    !0 >> (32 - bits % 32) % 32
}

impl BitVec {
    /// Applies the given operation to the blocks of self and other, and sets
    /// self to be the result. This relies on the caller not to corrupt the
    /// last word.
    #[inline]
    fn process<F>(&mut self, other: &BitVec, mut op: F) -> bool where F: FnMut(u32, u32) -> u32 {
        assert_eq!(self.len(), other.len());
        // This could theoretically be a `debug_assert!`.
        assert_eq!(self.storage.len(), other.storage.len());
        let mut changed_bits = 0;
        for (a, b) in self.blocks_mut().zip(other.blocks()) {
            let w = op(*a, b);
            changed_bits |= *a ^ w;
            *a = w;
        }
        changed_bits != 0
    }

    /// Iterator over mutable refs to  the underlying blocks of data.
    fn blocks_mut(&mut self) -> MutBlocks {
        // (2)
        self.storage.iter_mut()
    }

    /// Iterator over the underlying blocks of data
    fn blocks(&self) -> Blocks {
        // (2)
        self.storage.iter().cloned()
    }

    /// An operation might screw up the unused bits in the last block of the
    /// `BitVec`. As per (3), it's assumed to be all 0s. This method fixes it up.
    fn fix_last_block(&mut self) {
        let extra_bits = self.len() % 32;
        if extra_bits > 0 {
            let mask = (1 << extra_bits) - 1;
            let storage_len = self.storage.len();
            self.storage[storage_len - 1] &= mask;
        }
    }

    /// Creates an empty `BitVec`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    /// let mut bv = BitVec::new();
    /// ```
    
    pub fn new() -> BitVec {
        BitVec { storage: Vec::new(), nbits: 0 }
    }

    /// Creates a `BitVec` that holds `nbits` elements, setting each element
    /// to `bit`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let bv = BitVec::from_elem(10, false);
    /// assert_eq!(bv.len(), 10);
    /// for x in &bv {
    ///     assert_eq!(x, false);
    /// }
    /// ```
    pub fn from_elem(nbits: usize, bit: bool) -> BitVec {
        let nblocks = blocks_for_bits(nbits);
        let mut bit_vec = BitVec {
            storage: repeat(if bit { !0 } else { 0 }).take(nblocks).collect(),
            nbits: nbits
        };
        bit_vec.fix_last_block();
        bit_vec
    }

    /// Constructs a new, empty `BitVec` with the specified capacity.
    ///
    /// The bitvector will be able to hold at least `capacity` bits without
    /// reallocating. If `capacity` is 0, it will not allocate.
    ///
    /// It is important to note that this function does not specify the
    /// *length* of the returned bitvector, but only the *capacity*.
    
    pub fn with_capacity(nbits: usize) -> BitVec {
        BitVec {
            storage: Vec::with_capacity(blocks_for_bits(nbits)),
            nbits: 0,
        }
    }

    /// Transforms a byte-vector into a `BitVec`. Each byte becomes eight bits,
    /// with the most significant bits of each byte coming first. Each
    /// bit becomes `true` if equal to 1 or `false` if equal to 0.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let bv = BitVec::from_bytes(&[0b10100000, 0b00010010]);
    /// assert!(bv.eq_vec(&[true, false, true, false,
    ///                     false, false, false, false,
    ///                     false, false, false, true,
    ///                     false, false, true, false]));
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> BitVec {
        let len = bytes.len().checked_mul(8).expect("capacity overflow");
        let mut bit_vec = BitVec::with_capacity(len);
        let complete_words = bytes.len() / 4;
        let extra_bytes = bytes.len() % 4;

        bit_vec.nbits = len;

        for i in 0..complete_words {
            bit_vec.storage.push(
                ((reverse_bits(bytes[i * 4 + 0]) as u32) << 0) |
                ((reverse_bits(bytes[i * 4 + 1]) as u32) << 8) |
                ((reverse_bits(bytes[i * 4 + 2]) as u32) << 16) |
                ((reverse_bits(bytes[i * 4 + 3]) as u32) << 24)
            );
        }

        if extra_bytes > 0 {
            let mut last_word = 0;
            for (i, &byte) in bytes[complete_words*4..].iter().enumerate() {
                last_word |= (reverse_bits(byte) as u32) << (i * 8);
            }
            bit_vec.storage.push(last_word);
        }

        bit_vec
    }

    /// Creates a `BitVec` of the specified length where the value at each index
    /// is `f(index)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let bv = BitVec::from_fn(5, |i| { i % 2 == 0 });
    /// assert!(bv.eq_vec(&[true, false, true, false, true]));
    /// ```
    pub fn from_fn<F>(len: usize, mut f: F) -> BitVec where F: FnMut(usize) -> bool {
        let mut bit_vec = BitVec::from_elem(len, false);
        for i in 0..len {
            bit_vec.set(i, f(i));
        }
        bit_vec
    }

    /// Retrieves the value at index `i`, or `None` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let bv = BitVec::from_bytes(&[0b01100000]);
    /// assert_eq!(bv.get(0), Some(false));
    /// assert_eq!(bv.get(1), Some(true));
    /// assert_eq!(bv.get(100), None);
    ///
    /// // Can also use array indexing
    /// assert_eq!(bv[1], true);
    /// ```
    #[inline]
    
    pub fn get(&self, i: usize) -> Option<bool> {
        if i >= self.nbits {
            return None;
        }
        let w = i / 32;
        let b = i % 32;
        self.storage.get(w).map(|&block|
            (block & (1 << b)) != 0
        )
    }

    /// Sets the value of a bit at an index `i`.
    ///
    /// # Panics
    ///
    /// Panics if `i` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::from_elem(5, false);
    /// bv.set(3, true);
    /// assert_eq!(bv[3], true);
    /// ```
    #[inline]
    pub fn set(&mut self, i: usize, x: bool) {
        assert!(i < self.nbits);
        let w = i / 32;
        let b = i % 32;
        let flag = 1 << b;
        let val = if x { self.storage[w] | flag }
                  else { self.storage[w] & !flag };
        self.storage[w] = val;
    }

    /// Sets all bits to 1.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let before = 0b01100000;
    /// let after  = 0b11111111;
    ///
    /// let mut bv = BitVec::from_bytes(&[before]);
    /// bv.set_all();
    /// assert_eq!(bv, BitVec::from_bytes(&[after]));
    /// ```
    #[inline]
    pub fn set_all(&mut self) {
        for w in &mut self.storage { *w = !0; }
        self.fix_last_block();
    }

    /// Flips all bits.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let before = 0b01100000;
    /// let after  = 0b10011111;
    ///
    /// let mut bv = BitVec::from_bytes(&[before]);
    /// bv.negate();
    /// assert_eq!(bv, BitVec::from_bytes(&[after]));
    /// ```
    #[inline]
    pub fn negate(&mut self) {
        for w in &mut self.storage { *w = !*w; }
        self.fix_last_block();
    }

    /// Calculates the union of two bitvectors. This acts like the bitwise `or`
    /// function.
    ///
    /// Sets `self` to the union of `self` and `other`. Both bitvectors must be
    /// the same length. Returns `true` if `self` changed.
    ///
    /// # Panics
    ///
    /// Panics if the bitvectors are of different lengths.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let a   = 0b01100100;
    /// let b   = 0b01011010;
    /// let res = 0b01111110;
    ///
    /// let mut a = BitVec::from_bytes(&[a]);
    /// let b = BitVec::from_bytes(&[b]);
    ///
    /// assert!(a.union(&b));
    /// assert_eq!(a, BitVec::from_bytes(&[res]));
    /// ```
    #[inline]
    pub fn union(&mut self, other: &BitVec) -> bool {
        self.process(other, |w1, w2| w1 | w2)
    }

    /// Calculates the intersection of two bitvectors. This acts like the
    /// bitwise `and` function.
    ///
    /// Sets `self` to the intersection of `self` and `other`. Both bitvectors
    /// must be the same length. Returns `true` if `self` changed.
    ///
    /// # Panics
    ///
    /// Panics if the bitvectors are of different lengths.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let a   = 0b01100100;
    /// let b   = 0b01011010;
    /// let res = 0b01000000;
    ///
    /// let mut a = BitVec::from_bytes(&[a]);
    /// let b = BitVec::from_bytes(&[b]);
    ///
    /// assert!(a.intersect(&b));
    /// assert_eq!(a, BitVec::from_bytes(&[res]));
    /// ```
    #[inline]
    pub fn intersect(&mut self, other: &BitVec) -> bool {
        self.process(other, |w1, w2| w1 & w2)
    }

    /// Calculates the difference between two bitvectors.
    ///
    /// Sets each element of `self` to the value of that element minus the
    /// element of `other` at the same index. Both bitvectors must be the same
    /// length. Returns `true` if `self` changed.
    ///
    /// # Panics
    ///
    /// Panics if the bitvectors are of different length.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let a   = 0b01100100;
    /// let b   = 0b01011010;
    /// let a_b = 0b00100100; // a - b
    /// let b_a = 0b00011010; // b - a
    ///
    /// let mut bva = BitVec::from_bytes(&[a]);
    /// let bvb = BitVec::from_bytes(&[b]);
    ///
    /// assert!(bva.difference(&bvb));
    /// assert_eq!(bva, BitVec::from_bytes(&[a_b]));
    ///
    /// let bva = BitVec::from_bytes(&[a]);
    /// let mut bvb = BitVec::from_bytes(&[b]);
    ///
    /// assert!(bvb.difference(&bva));
    /// assert_eq!(bvb, BitVec::from_bytes(&[b_a]));
    /// ```
    #[inline]
    pub fn difference(&mut self, other: &BitVec) -> bool {
        self.process(other, |w1, w2| w1 & !w2)
    }

    /// Returns `true` if all bits are 1.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::from_elem(5, true);
    /// assert_eq!(bv.all(), true);
    ///
    /// bv.set(1, false);
    /// assert_eq!(bv.all(), false);
    /// ```
    pub fn all(&self) -> bool {
        let mut last_word = !0;
        // Check that every block but the last is all-ones...
        self.blocks().all(|elem| {
            let tmp = last_word;
            last_word = elem;
            tmp == !0
        // and then check the last one has enough ones
        }) && (last_word == mask_for_bits(self.nbits))
    }

    /// Returns an iterator over the elements of the vector in order.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let bv = BitVec::from_bytes(&[0b01110100, 0b10010010]);
    /// assert_eq!(bv.iter().filter(|x| *x).count(), 7);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter {
        Iter { bit_vec: self, next_idx: 0, end_idx: self.nbits }
    }

    /// Moves all bits from `other` into `Self`, leaving `other` empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec, append)]
    /// use std::collections::BitVec;
    ///
    /// let mut a = BitVec::from_bytes(&[0b10000000]);
    /// let mut b = BitVec::from_bytes(&[0b01100001]);
    ///
    /// a.append(&mut b);
    ///
    /// assert_eq!(a.len(), 16);
    /// assert_eq!(b.len(), 0);
    /// assert!(a.eq_vec(&[true, false, false, false, false, false, false, false,
    ///                    false, true, true, false, false, false, false, true]));
    /// ```
    pub fn append(&mut self, other: &mut Self) {
        let b = self.len() % 32;

        self.nbits += other.len();
        other.nbits = 0;

        if b == 0 {
            self.storage.append(&mut other.storage);
        } else {
            self.storage.reserve(other.storage.len());

            for block in other.storage.drain(..) {
                *(self.storage.last_mut().unwrap()) |= block << b;
                self.storage.push(block >> (32 - b));
            }
        }
    }

    /// Splits the `BitVec` into two at the given bit,
    /// retaining the first half in-place and returning the second one.
    ///
    /// # Panics
    ///
    /// Panics if `at` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec, split_off)]
    /// use std::collections::BitVec;
    /// let mut a = BitVec::new();
    /// a.push(true);
    /// a.push(false);
    /// a.push(false);
    /// a.push(true);
    ///
    /// let b = a.split_off(2);
    ///
    /// assert_eq!(a.len(), 2);
    /// assert_eq!(b.len(), 2);
    /// assert!(a.eq_vec(&[true, false]));
    /// assert!(b.eq_vec(&[false, true]));
    /// ```
    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(at <= self.len(), "`at` out of bounds");

        let mut other = BitVec::new();

        if at == 0 {
            swap(self, &mut other);
            return other;
        } else if at == self.len() {
            return other;
        }

        let w = at / 32;
        let b = at % 32;
        other.nbits = self.nbits - at;
        self.nbits = at;
        if b == 0 {
            // Split at block boundary
            other.storage = self.storage.split_off(w);
        } else {
            other.storage.reserve(self.storage.len() - w);

            {
                let mut iter = self.storage[w..].iter();
                let mut last = *iter.next().unwrap();
                for &cur in iter {
                    other.storage.push((last >> b) | (cur << (32 - b)));
                    last = cur;
                }
                other.storage.push(last >> b);
            }

            self.storage.truncate(w+1);
            self.fix_last_block();
        }

        other
    }

    /// Returns `true` if all bits are 0.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::from_elem(10, false);
    /// assert_eq!(bv.none(), true);
    ///
    /// bv.set(3, true);
    /// assert_eq!(bv.none(), false);
    /// ```
    pub fn none(&self) -> bool {
        self.blocks().all(|w| w == 0)
    }

    /// Returns `true` if any bit is 1.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::from_elem(10, false);
    /// assert_eq!(bv.any(), false);
    ///
    /// bv.set(3, true);
    /// assert_eq!(bv.any(), true);
    /// ```
    #[inline]
    pub fn any(&self) -> bool {
        !self.none()
    }

    /// Organises the bits into bytes, such that the first bit in the
    /// `BitVec` becomes the high-order bit of the first byte. If the
    /// size of the `BitVec` is not a multiple of eight then trailing bits
    /// will be filled-in with `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::from_elem(3, true);
    /// bv.set(1, false);
    ///
    /// assert_eq!(bv.to_bytes(), [0b10100000]);
    ///
    /// let mut bv = BitVec::from_elem(9, false);
    /// bv.set(2, true);
    /// bv.set(8, true);
    ///
    /// assert_eq!(bv.to_bytes(), [0b00100000, 0b10000000]);
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        fn bit(bit_vec: &BitVec, byte: usize, bit: usize) -> u8 {
            let offset = byte * 8 + bit;
            if offset >= bit_vec.nbits {
                0
            } else {
                (bit_vec[offset] as u8) << (7 - bit)
            }
        }

        let len = self.nbits/8 +
                  if self.nbits % 8 == 0 { 0 } else { 1 };
        (0..len).map(|i|
            bit(self, i, 0) |
            bit(self, i, 1) |
            bit(self, i, 2) |
            bit(self, i, 3) |
            bit(self, i, 4) |
            bit(self, i, 5) |
            bit(self, i, 6) |
            bit(self, i, 7)
        ).collect()
    }

    /// Compares a `BitVec` to a slice of `bool`s.
    /// Both the `BitVec` and slice must have the same length.
    ///
    /// # Panics
    ///
    /// Panics if the `BitVec` and slice are of different length.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let bv = BitVec::from_bytes(&[0b10100000]);
    ///
    /// assert!(bv.eq_vec(&[true, false, true, false,
    ///                     false, false, false, false]));
    /// ```
    pub fn eq_vec(&self, v: &[bool]) -> bool {
        assert_eq!(self.nbits, v.len());
        self.iter().eq(v.iter().cloned())
    }

    /// Shortens a `BitVec`, dropping excess elements.
    ///
    /// If `len` is greater than the vector's current length, this has no
    /// effect.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::from_bytes(&[0b01001011]);
    /// bv.truncate(2);
    /// assert!(bv.eq_vec(&[false, true]));
    /// ```
    pub fn truncate(&mut self, len: usize) {
        if len < self.len() {
            self.nbits = len;
            // This fixes (2).
            self.storage.truncate(blocks_for_bits(len));
            self.fix_last_block();
        }
    }

    /// Reserves capacity for at least `additional` more bits to be inserted in the given
    /// `BitVec`. The collection may reserve more space to avoid frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::from_elem(3, false);
    /// bv.reserve(10);
    /// assert_eq!(bv.len(), 3);
    /// assert!(bv.capacity() >= 13);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        let desired_cap = self.len().checked_add(additional).expect("capacity overflow");
        let storage_len = self.storage.len();
        if desired_cap > self.capacity() {
            self.storage.reserve(blocks_for_bits(desired_cap) - storage_len);
        }
    }

    /// Reserves the minimum capacity for exactly `additional` more bits to be inserted in the
    /// given `BitVec`. Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it requests. Therefore
    /// capacity can not be relied upon to be precisely minimal. Prefer `reserve` if future
    /// insertions are expected.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::from_elem(3, false);
    /// bv.reserve(10);
    /// assert_eq!(bv.len(), 3);
    /// assert!(bv.capacity() >= 13);
    /// ```
    pub fn reserve_exact(&mut self, additional: usize) {
        let desired_cap = self.len().checked_add(additional).expect("capacity overflow");
        let storage_len = self.storage.len();
        if desired_cap > self.capacity() {
            self.storage.reserve_exact(blocks_for_bits(desired_cap) - storage_len);
        }
    }

    /// Returns the capacity in bits for this bit vector. Inserting any
    /// element less than this amount will not trigger a resizing.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::new();
    /// bv.reserve(10);
    /// assert!(bv.capacity() >= 10);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.storage.capacity().checked_mul(32).unwrap_or(usize::MAX)
    }

    /// Grows the `BitVec` in-place, adding `n` copies of `value` to the `BitVec`.
    ///
    /// # Panics
    ///
    /// Panics if the new len overflows a `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::from_bytes(&[0b01001011]);
    /// bv.grow(2, true);
    /// assert_eq!(bv.len(), 10);
    /// assert_eq!(bv.to_bytes(), [0b01001011, 0b11000000]);
    /// ```
    pub fn grow(&mut self, n: usize, value: bool) {
        // Note: we just bulk set all the bits in the last word in this fn in multiple places
        // which is technically wrong if not all of these bits are to be used. However, at the end
        // of this fn we call `fix_last_block` at the end of this fn, which should fix this.

        let new_nbits = self.nbits.checked_add(n).expect("capacity overflow");
        let new_nblocks = blocks_for_bits(new_nbits);
        let full_value = if value { !0 } else { 0 };

        // Correct the old tail word, setting or clearing formerly unused bits
        let num_cur_blocks = blocks_for_bits(self.nbits);
        if self.nbits % 32 > 0 {
            let mask = mask_for_bits(self.nbits);
            if value {
                self.storage[num_cur_blocks - 1] |= !mask;
            } else {
                // Extra bits are already zero by invariant.
            }
        }

        // Fill in words after the old tail word
        let stop_idx = cmp::min(self.storage.len(), new_nblocks);
        for idx in num_cur_blocks..stop_idx {
            self.storage[idx] = full_value;
        }

        // Allocate new words, if needed
        if new_nblocks > self.storage.len() {
            let to_add = new_nblocks - self.storage.len();
            self.storage.extend(repeat(full_value).take(to_add));
        }

        // Adjust internal bit count
        self.nbits = new_nbits;

        self.fix_last_block();
    }

    /// Removes the last bit from the BitVec, and returns it. Returns None if the BitVec is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::from_bytes(&[0b01001001]);
    /// assert_eq!(bv.pop(), Some(true));
    /// assert_eq!(bv.pop(), Some(false));
    /// assert_eq!(bv.len(), 6);
    /// ```
    pub fn pop(&mut self) -> Option<bool> {
        if self.is_empty() {
            None
        } else {
            let i = self.nbits - 1;
            let ret = self[i];
            // (3)
            self.set(i, false);
            self.nbits = i;
            if self.nbits % 32 == 0 {
                // (2)
                self.storage.pop();
            }
            Some(ret)
        }
    }

    /// Pushes a `bool` onto the end.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![feature(bitvec)]
    /// use std::collections::BitVec;
    ///
    /// let mut bv = BitVec::new();
    /// bv.push(true);
    /// bv.push(false);
    /// assert!(bv.eq_vec(&[true, false]));
    /// ```
    
    pub fn push(&mut self, elem: bool) {
        if self.nbits % 32 == 0 {
            self.storage.push(0);
        }
        let insert_pos = self.nbits;
        self.nbits = self.nbits.checked_add(1).expect("Capacity overflow");
        self.set(insert_pos, elem);
    }

    /// Returns the total number of bits in this vector
    #[inline]
    
    pub fn len(&self) -> usize { self.nbits }

    /// Returns true if there are no bits in this vector
    #[inline]
    
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Clears all bits in this vector.
    #[inline]
    
    pub fn clear(&mut self) {
        for w in &mut self.storage { *w = 0; }
    }
}


impl Default for BitVec {
    #[inline]
    fn default() -> BitVec { BitVec::new() }
}


impl FromIterator<bool> for BitVec {
    fn from_iter<I: IntoIterator<Item=bool>>(iter: I) -> BitVec {
        let mut ret = BitVec::new();
        ret.extend(iter);
        ret
    }
}


impl Extend<bool> for BitVec {
    #[inline]
    fn extend<I: IntoIterator<Item=bool>>(&mut self, iterable: I) {
        let iterator = iterable.into_iter();
        let (min, _) = iterator.size_hint();
        self.reserve(min);
        for element in iterator {
            self.push(element)
        }
    }
}


impl<'a> Extend<&'a bool> for BitVec {
    fn extend<I: IntoIterator<Item=&'a bool>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}


impl Clone for BitVec {
    #[inline]
    fn clone(&self) -> BitVec {
        BitVec { storage: self.storage.clone(), nbits: self.nbits }
    }

    #[inline]
    fn clone_from(&mut self, source: &BitVec) {
        self.nbits = source.nbits;
        self.storage.clone_from(&source.storage);
    }
}


impl fmt::Debug for BitVec {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for bit in self {
            try!(write!(fmt, "{}", if bit { 1 } else { 0 }));
        }
        Ok(())
    }
}


impl hash::Hash for BitVec {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.nbits.hash(state);
        for elem in self.blocks() {
            elem.hash(state);
        }
    }
}


impl cmp::PartialEq for BitVec {
    #[inline]
    fn eq(&self, other: &BitVec) -> bool {
        if self.nbits != other.nbits {
            return false;
        }
        self.blocks().zip(other.blocks()).all(|(w1, w2)| w1 == w2)
    }
}


impl cmp::Eq for BitVec {}

/// An iterator for `BitVec`.

#[derive(Clone)]
pub struct Iter<'a> {
    bit_vec: &'a BitVec,
    next_idx: usize,
    end_idx: usize,
}


impl<'a> Iterator for Iter<'a> {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<bool> {
        if self.next_idx != self.end_idx {
            let idx = self.next_idx;
            self.next_idx += 1;
            Some(self.bit_vec[idx])
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.end_idx - self.next_idx;
        (rem, Some(rem))
    }
}


impl<'a> DoubleEndedIterator for Iter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<bool> {
        if self.next_idx != self.end_idx {
            self.end_idx -= 1;
            Some(self.bit_vec[self.end_idx])
        } else {
            None
        }
    }
}


impl<'a> ExactSizeIterator for Iter<'a> {}


impl<'a> IntoIterator for &'a BitVec {
    type Item = bool;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Iter<'a> {
        self.iter()
    }
}
