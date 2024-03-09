use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::name::QName;
use quick_xml::{Reader, Writer};

use crate::error::{DecodingError, EncodingError, Result};

pub(crate) trait ReaderExt {
    fn expect_tag(&mut self, end: QName) -> Result<()>;
}

impl<'a> ReaderExt for Reader<&'a [u8]> {
    fn expect_tag(&mut self, end: QName) -> Result<()> {
        loop {
            match self.read_event() {
                // TODO: this isn't exactly right, but it's good enough for now.
                Ok(Event::Decl(ref _d)) => continue,
                Ok(Event::Start(ref e)) => {
                    if e.name() != end {
                        return Err(DecodingError::UnexpectedTag(
                            String::from_utf8_lossy(e.name().into_inner()).into(),
                            String::from_utf8_lossy(end.into_inner()).into(),
                        )
                        .into());
                    }

                    break;
                }
                Ok(_e) => {
                    return Err(DecodingError::UnexpectedEvent(
                        //e,
                        String::from_utf8_lossy(end.into_inner()).into(),
                    )
                    .into());
                }
                Err(e) => {
                    return Err(DecodingError::UnexpectedError(
                        e.into(),
                        String::from_utf8_lossy(end.into_inner()).into(),
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
    fn write_tag(&mut self, tag: &str, text: &str) -> Result<()> {
        self.write_start_tag(tag)?;
        self.write_text(text)?;
        self.write_end_tag(tag)?;
        Ok(())
    }

    fn write_safe_tag(&mut self, tag: &str, text: &str) -> Result<()> {
        self.write_start_tag(tag)?;
        self.write_safe_text(text)?;
        self.write_end_tag(tag)?;
        Ok(())
    }

    // Building blocks
    fn write_decl(&mut self) -> Result<()>;
    fn write_start_tag(&mut self, tag: &str) -> Result<()>;
    fn write_end_tag(&mut self, tag: &str) -> Result<()>;
    fn write_text(&mut self, text: &str) -> Result<()>;
    fn write_safe_text(&mut self, text: &str) -> Result<()>;
}

impl<W> WriterExt for Writer<W>
where
    W: std::io::Write,
{
    fn write_decl(&mut self) -> Result<()> {
        self.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
            .map_err(EncodingError::from)?;
        Ok(())
    }

    fn write_start_tag(&mut self, tag: &str) -> Result<()> {
        self.write_event(Event::Start(BytesStart::new(tag)))
            .map_err(EncodingError::from)?;
        Ok(())
    }

    fn write_end_tag(&mut self, tag: &str) -> Result<()> {
        self.write_event(Event::End(BytesEnd::new(tag)))
            .map_err(EncodingError::from)?;
        Ok(())
    }
    fn write_text(&mut self, text: &str) -> Result<()> {
        self.write_event(Event::Text(BytesText::new(text)))
            .map_err(EncodingError::from)?;
        Ok(())
    }
    fn write_safe_text(&mut self, text: &str) -> Result<()> {
        self.write_event(Event::Text(BytesText::from_escaped(text)))
            .map_err(EncodingError::from)?;
        Ok(())
    }
}
