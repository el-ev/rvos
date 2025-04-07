pub struct Bitmap<const T: usize> {
    bitmap: [usize; T],
}

impl<const T: usize> Bitmap<T> {
    // pub const BITS: usize = T * usize::BITS as usize;

    pub const fn new() -> Self {
        Self { bitmap: [0; T] }
    }

    pub fn set(&mut self, pos: usize, count: usize) {
        // assert!(pos < T * usize::BITS as usize);
        // let index = pos / usize::BITS as usize;
        // let offset = pos % usize::BITS as usize;
        // self.bitmap[index] |= 1 << offset;
        assert!(pos < T * usize::BITS as usize && pos + count <= T * usize::BITS as usize);
        let index = pos / usize::BITS as usize;
        let offset = pos % usize::BITS as usize;
        let mut remaining = count;
        let mut curidx = index;
        let mut curoff = offset;

        while remaining > 0 {
            let bits_in_cur_word = usize::BITS as usize - curoff;
            let bits_to_set = bits_in_cur_word.min(remaining);

            self.bitmap[curidx] |= ((1 << bits_to_set) - 1) << curoff;

            remaining -= bits_to_set;
            curidx += 1;
            curoff = 0;
        }
    }

    pub fn clear(&mut self, pos: usize, count: usize) {
        assert!(pos < T * usize::BITS as usize && pos + count <= T * usize::BITS as usize);
        // let index = pos / usize::BITS as usize;
        // let offset = pos % usize::BITS as usize;
        // self.bitmap[index] &= !(1 << offset);
        let index = pos / usize::BITS as usize;
        let offset = pos % usize::BITS as usize;
        let mut remaining = count;
        let mut curidx = index;
        let mut curoff = offset;

        while remaining > 0 {
            let bits_in_cur_word = usize::BITS as usize - curoff;
            let bits_to_clear = bits_in_cur_word.min(remaining);

            self.bitmap[curidx] &= !(((1 << bits_to_clear) - 1) << curoff);

            remaining -= bits_to_clear;
            curidx += 1;
            curoff = 0;
        }
    }

    // pub fn get(&self, pos: usize) -> bool {
    //     assert!(pos < T);
    //     let index = pos / usize::BITS as usize;
    //     let offset = pos % usize::BITS as usize;
    //     (self.bitmap[index] & (1 << offset)) != 0
    // }

    pub fn first_zero(&self) -> Option<usize> {
        for (i, &word) in self.bitmap.iter().enumerate() {
            if word != usize::MAX {
                let offset = word.trailing_ones() as usize;
                return Some(i * usize::BITS as usize + offset);
            }
        }
        None
    }

    pub fn find_contiguous(&self, size: usize, align: usize) -> Option<usize> {
        debug_assert!(align.is_power_of_two() && (align >= size || align == 0));
        if align == 0 {
            return self.find_contiguous(size, 1);
        }
        
        let mut count = 0;
        let mut start_pos = None;
        
        for (i, &word) in self.bitmap.iter().enumerate() {
            let base = i * usize::BITS as usize;
            
            if word == usize::MAX {
                count = 0;
                start_pos = None;
                continue;
            }
            
            let mut mask = 1;
            for bit in 0..usize::BITS as usize {
                let pos = base + bit;
                
                if word & mask == 0 {
                    if count == 0 && pos % align == 0 {
                        start_pos = Some(pos);
                        count = 1;
                    } else if count > 0 {
                        count += 1;
                    }
                    
                    if count >= size {
                        return start_pos;
                    }
                } else {
                    count = 0;
                    start_pos = None;
                }
                
                mask <<= 1;
            }
        }
        
        None
    }

    // pub fn first_one(&self) -> Option<usize> {
    //     for (i, &word) in self.bitmap.iter().enumerate() {
    //         if word != 0 {
    //             let offset = word.trailing_zeros() as usize;
    //             return Some(i * usize::BITS as usize + offset);
    //         }
    //     }
    //     None
    // }
}
