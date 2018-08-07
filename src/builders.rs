#[derive(Clone, Default, Debug)]
pub struct Utf8CharBuilder {
    bytes: [u8; 4],
    len: usize,
}
impl Utf8CharBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn append(&mut self, byte: u8) -> Option<Result<Utf8Char,InvalidUtf8___>> {
        
    }
    pub fn finish(self) -> Option<Result<Utf8Char,InvalidUtf8___>> {
        
    }
}

#[derive(Clone, Default, Debug)]
pub struct Utf16CharBuilder {
    prev: Option<u16>
}
impl Utf16CharBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn append(&mut self, unit: u16) -> Option<Result<Utf16Char,InvalidUtf16Tuple>> {
        let (ret, store) = match (self.prev, unit) {
            (None, 0x0000...0xd7ff) => (Some(Ok([unit, 0])), false),
            (None, 0xe000...0xffff) => (Some(Ok([unit, 0])), false),
            (None, 0xd800...0xdbff) => (None, true),
            (Some(first @ 0xd800...0xdbff), 0xdc00...0xdfff) => (Some(Ok([first,unit]})), false),
            (Some(prev @ 0x0000...0xd7ff), _) => (Some(Ok([prev,0])), true),
            (Some(prev @ 0xe000...0xffff), _) => (Some(Ok([prev,0])), true),
            (None, 0xdc00...0xdfff) => (Some(Err(FirstIsTrailingSurrogate)), false),
            (Some(0xdc00...0xdfff), _) => (Some(Err(FirstIsTrailingSurrogate)), true),
            (Some(0xd800...0xdbff), _) => (Some(Err(InvalidSecond)), true)
        };
        self.prev = if store {Some(unit)} else {None};
        // makes the above code more compact, hope the compiler recognize it as the noop it is.
        ret.map(|some| some.map(|units| Utf16Char{units: units} ) )
    }
    pub fn finish(self) -> Option<Result<Utf16Char,InvalidUtf16Tuple>> {
        match self.prev {
            None => None,
            Some(u @ 0x0000...0xd7ff) => Some(Ok(Utf16CharBuilder{units: [u,0]})),
            Some(u @ 0xe000...0xdfff) => Some(Ok(Utf16CharBuilder{units: [u,0]})),
            Some(_ @ 0xde00...0xdfff) => Some(Err(FirstIsTrailingSurrogate)),
            Some(_ @ 0xd800...0xdbff) => Some(Err(MissingSecond)),
        }
    }
}