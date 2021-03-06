use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};

use crate::error::{EncodingError, ParseError, Result};

mod map;
mod seq;
mod value;

pub use map::{MapDeserializer, MapSerializer};
pub use seq::{SeqDeserializer, SeqSerializer};
pub use value::{Deserializer as ValueDeserializer, Serializer as ValueSerializer};

pub(crate) trait ReaderExt {
    fn expect_tag(&mut self, end: &[u8], buf: &mut Vec<u8>) -> Result<()>;
}

impl<B> ReaderExt for Reader<B>
where
    B: std::io::BufRead,
{
    fn expect_tag(&mut self, end: &[u8], buf: &mut Vec<u8>) -> Result<()> {
        loop {
            match self.read_event(buf) {
                // TODO: this isn't exactly right, but it's good enough for now.
                Ok(Event::Decl(ref _d)) => continue,
                Ok(Event::Start(ref e)) => {
                    if e.name() != end {
                        return Err(ParseError::UnexpectedTag(
                            String::from_utf8_lossy(e.name()).into(),
                            String::from_utf8_lossy(end).into(),
                        )
                        .into());
                    }

                    break;
                }
                Ok(_e) => {
                    return Err(ParseError::UnexpectedEvent(
                        //e,
                        String::from_utf8_lossy(end).into(),
                    )
                    .into());
                }
                Err(e) => {
                    return Err(ParseError::UnexpectedError(
                        e.into(),
                        String::from_utf8_lossy(end).into(),
                    )
                    .into())
                }
            };
        }

        Ok(())
    }
}

pub(crate) trait WriterExt {
    // High level functions
    fn write_tag(&mut self, tag: &[u8], text: &str) -> Result<()> {
        self.write_start_tag(tag)?;
        self.write_text(text)?;
        self.write_end_tag(tag)?;
        Ok(())
    }

    fn write_safe_tag(&mut self, tag: &[u8], text: &str) -> Result<()> {
        self.write_start_tag(tag)?;
        self.write_safe_text(text)?;
        self.write_end_tag(tag)?;
        Ok(())
    }

    // Building blocks
    fn write_start_tag(&mut self, tag: &[u8]) -> Result<()>;
    fn write_end_tag(&mut self, tag: &[u8]) -> Result<()>;
    fn write_text(&mut self, text: &str) -> Result<()>;
    fn write_safe_text(&mut self, text: &str) -> Result<()>;
}

impl<W> WriterExt for Writer<W>
where
    W: std::io::Write,
{
    fn write_start_tag(&mut self, tag: &[u8]) -> Result<()> {
        self.write_event(Event::Start(BytesStart::borrowed_name(tag)))
            .map_err(EncodingError::from)?;
        Ok(())
    }

    fn write_end_tag(&mut self, tag: &[u8]) -> Result<()> {
        self.write_event(Event::End(BytesEnd::borrowed(tag)))
            .map_err(EncodingError::from)?;
        Ok(())
    }
    fn write_text(&mut self, text: &str) -> Result<()> {
        self.write_event(Event::Text(BytesText::from_plain_str(text)))
            .map_err(EncodingError::from)?;
        Ok(())
    }
    fn write_safe_text(&mut self, text: &str) -> Result<()> {
        self.write_event(Event::Text(BytesText::from_escaped_str(text)))
            .map_err(EncodingError::from)?;
        Ok(())
    }
}
