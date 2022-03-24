pub trait Peek<T> {
    fn peek(&self) -> Option<&T>;
    unsafe fn peek_unchecked(&self) -> &T;
}

impl<T> Peek<T> for Vec<T> {
    fn peek(&self) -> Option<&T> {
        self.as_slice().get(self.len() - 1)
    }
    unsafe fn peek_unchecked(&self) -> &T {
        debug_assert!(self.peek().is_some());
        self.as_slice().get_unchecked(self.len() - 1)
    }
}

pub trait Pop<T> {
    unsafe fn pop_unchecked(&mut self) -> T;
}

impl<T> Pop<T> for Vec<T> {
    /// Pops the top element of the vector and returns it without doing any bounds/sanity checking. Causes undefined behaviour if self.is_empty() || self.buff.cap == 0
    unsafe fn pop_unchecked(&mut self) -> T {
        self.set_len(self.len() - 1);
        std::ptr::read(self.as_ptr().add(self.len()))
    }
}

pub trait PopChar {
    fn pop_char(&mut self) -> Option<char>;
}

impl PopChar for &str {
    /// Pops the first char from the string and returns it. returns None if the string is empty.

    fn pop_char(&mut self) -> Option<char> {
        let top = self.chars().next();
        match top {
            None => None,
            Some(v) => {
                *self = &self[v.len_utf8()..];
                Some(v)
            }
        }
    }
}

pub unsafe trait PopByte {
    unsafe fn pop_byte(&mut self) -> Option<u8>;
}

unsafe impl PopByte for &str {
    /// Pops the first byte from the string and returns it. Will cause the string contents to no longer be valid utf8 if the top byte is not valid ASCII.
    unsafe fn pop_byte(&mut self) -> Option<u8> {
        let top = self.bytes().next();
        match top {
            None => None,
            Some(v) => {
                *self = &self[1..];
                Some(v)
            }
        }
    }
}
