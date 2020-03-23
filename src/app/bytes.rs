use crate::app::header::FixedSize;
use crate::app::parser::ParseError;
use crate::util::cursor::ReadCursor;

pub struct Bytes<'a> {
    pub value: &'a [u8],
}

pub struct RangedBytesIterator<'a> {
    cursor: ReadCursor<'a>,
    index: u16,
    size: usize,
}

pub struct PrefixedBytesIterator<'a, T>
where
    T: FixedSize,
{
    cursor: ReadCursor<'a>,
    size: usize,
    phantom: std::marker::PhantomData<T>,
}

impl<'a> Bytes<'a> {
    pub fn new(value: &'a [u8]) -> Self {
        Self { value }
    }
}

impl<'a> RangedBytesIterator<'a> {
    pub fn parse(
        variation: u8,
        start: u16,
        count: usize,
        cursor: &mut ReadCursor<'a>,
    ) -> Result<impl Iterator<Item = (Bytes<'a>, u16)>, ParseError> {
        if variation == 0 {
            return Err(ParseError::ZeroLengthOctetData);
        }

        Ok(RangedBytesIterator {
            cursor: ReadCursor::new(cursor.read_bytes(variation as usize * count)?),
            index: start,
            size: variation as usize,
        })
    }
}

impl<'a, T> PrefixedBytesIterator<'a, T>
where
    T: FixedSize,
{
    pub fn parse(
        variation: u8,
        count: usize,
        cursor: &mut ReadCursor<'a>,
    ) -> Result<impl Iterator<Item = (Bytes<'a>, T)>, ParseError> {
        if variation == 0 {
            return Err(ParseError::ZeroLengthOctetData);
        }

        let size = (variation as usize + T::SIZE as usize) * count;

        Ok(PrefixedBytesIterator {
            cursor: ReadCursor::new(cursor.read_bytes(size)?),
            size: variation as usize,
            phantom: std::marker::PhantomData {},
        })
    }
}

impl<'a> Iterator for RangedBytesIterator<'a> {
    type Item = (Bytes<'a>, u16);

    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.read_bytes(self.size).ok().map(|b| {
            let index = self.index;
            self.index += 1;
            (Bytes::new(b), index)
        })
    }
}

impl<'a, T> Iterator for PrefixedBytesIterator<'a, T>
where
    T: FixedSize,
{
    type Item = (Bytes<'a>, T);

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = match self.cursor.read_bytes(self.size) {
            Ok(x) => x,
            _ => return None,
        };
        let index = match T::parse(&mut self.cursor) {
            Ok(x) => x,
            _ => return None,
        };

        Some((Bytes::new(bytes), index))
    }
}