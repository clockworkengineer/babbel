//! # Zero-Allocation Streaming Pull Parser (`XmlPullParser`)
//!
//! Designed specifically for resource-constrained microcontrollers and embedded systems.
//! Operates over raw byte slices (`&[u8]`) or string slices (`&str`) with zero dynamic heap allocation (`no_alloc`).

use crate::error::Result;

/// Borrowed key-value attribute pair for streaming SAX-style events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XmlPullAttribute<'a> {
    /// Borrowed attribute name slice.
    pub name: &'a str,
    /// Borrowed attribute value slice.
    pub value: &'a str,
}

/// Zero-allocation streaming event variant emitted by [`XmlPullParser`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlPullEvent<'a> {
    /// XML Declaration (e.g. `<?xml version="1.0" encoding="UTF-8"?>`).
    Declaration {
        /// Version string slice.
        version: &'a str,
        /// Optional encoding string slice.
        encoding: Option<&'a str>,
        /// Optional standalone flag.
        standalone: Option<bool>,
    },
    /// DOCTYPE definition (e.g. `<!DOCTYPE html SYSTEM "...">`).
    DocType {
        /// Document type root name.
        name: &'a str,
        /// Optional PUBLIC identifier.
        public_id: Option<&'a str>,
        /// Optional SYSTEM identifier.
        system_id: Option<&'a str>,
    },
    /// Opening element tag (e.g. `<book category="web">`).
    StartElement {
        /// Element tag name.
        name: &'a str,
        /// Raw attribute text slice within element tag.
        attr_raw: &'a str,
    },
    /// Closing element tag (e.g. `</book>`).
    EndElement {
        /// Element tag name.
        name: &'a str,
    },
    /// Text node content slice.
    Text(&'a str),
    /// Comment content slice.
    Comment(&'a str),
    /// CDATA section content slice.
    CData(&'a str),
    /// Processing instruction (`<?target data?>`).
    ProcessingInstruction {
        /// PI target name.
        target: &'a str,
        /// PI data payload.
        data: &'a str,
    },
    /// End of XML document stream.
    EndDocument,
}

impl<'a> XmlPullEvent<'a> {
    /// Returns an iterator over tag attributes parsed on demand with zero allocations.
    pub fn attributes(&self) -> XmlPullAttributesIter<'a> {
        match self {
            Self::StartElement { attr_raw, .. } => XmlPullAttributesIter { raw: attr_raw },
            _ => XmlPullAttributesIter { raw: "" },
        }
    }
}

/// Zero-allocation iterator over tag attributes.
pub struct XmlPullAttributesIter<'a> {
    raw: &'a str,
}

impl<'a> Iterator for XmlPullAttributesIter<'a> {
    type Item = XmlPullAttribute<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let trimmed = self.raw.trim_start();
        if trimmed.is_empty() {
            return None;
        }

        if let Some((name_part, rest)) = trimmed.split_once('=') {
            let attr_name = name_part.trim();
            let val_part = rest.trim_start();
            if let Some(quote_char) = val_part.chars().next() {
                if quote_char == '"' || quote_char == '\'' {
                    if let Some(end_quote) = val_part[1..].find(quote_char) {
                        let attr_val = &val_part[1..1 + end_quote];
                        self.raw = &val_part[1 + end_quote + 1..];
                        return Some(XmlPullAttribute {
                            name: attr_name,
                            value: attr_val,
                        });
                    }
                }
            }
        }
        self.raw = "";
        None
    }
}

/// Zero-allocation streaming XML pull parser for embedded microcontrollers.
pub struct XmlPullParser<'a> {
    xml: &'a str,
    pos: usize,
    pending_end_element: Option<&'a str>,
}

impl<'a> XmlPullParser<'a> {
    /// Instantiates a new [`XmlPullParser`] over a borrowed string slice.
    pub fn new(xml: &'a str) -> Self {
        Self {
            xml,
            pos: 0,
            pending_end_element: None,
        }
    }

    /// Advances the parser and pulls the next [`XmlPullEvent`]. Returns `None` after EOF is reached.
    pub fn next_event(&mut self) -> Result<Option<XmlPullEvent<'a>>> {
        if let Some(tag_name) = self.pending_end_element.take() {
            return Ok(Some(XmlPullEvent::EndElement { name: tag_name }));
        }

        if self.pos >= self.xml.len() {
            return Ok(None);
        }

        let remaining = &self.xml[self.pos..];
        if remaining.is_empty() {
            return Ok(None);
        }

        if remaining.starts_with("<?xml") {
            if let Some(end_idx) = remaining.find("?>") {
                let content = &remaining[5..end_idx].trim();
                self.pos += end_idx + 2;

                let mut version = "1.0";
                let mut encoding = None;
                let mut standalone = None;

                let mut attrs_iter = XmlPullAttributesIter { raw: content };
                while let Some(attr) = attrs_iter.next() {
                    match attr.name {
                        "version" => version = attr.value,
                        "encoding" => encoding = Some(attr.value),
                        "standalone" => standalone = Some(attr.value == "yes"),
                        _ => {}
                    }
                }

                return Ok(Some(XmlPullEvent::Declaration {
                    version,
                    encoding,
                    standalone,
                }));
            }
        }

        if remaining.starts_with("<?") {
            if let Some(end_idx) = remaining.find("?>") {
                let content = &remaining[2..end_idx];
                self.pos += end_idx + 2;
                let (target, data) = content.split_once(char::is_whitespace).unwrap_or((content, ""));
                return Ok(Some(XmlPullEvent::ProcessingInstruction {
                    target: target.trim(),
                    data: data.trim(),
                }));
            }
        }

        if remaining.starts_with("<!--") {
            if let Some(end_idx) = remaining.find("-->") {
                let content = &remaining[4..end_idx];
                self.pos += end_idx + 3;
                return Ok(Some(XmlPullEvent::Comment(content)));
            }
        }

        if remaining.starts_with("<![CDATA[") {
            if let Some(end_idx) = remaining.find("]]>") {
                let content = &remaining[9..end_idx];
                self.pos += end_idx + 3;
                return Ok(Some(XmlPullEvent::CData(content)));
            }
        }

        if remaining.starts_with("<!DOCTYPE") {
            if let Some(end_idx) = remaining.find('>') {
                let content = remaining[9..end_idx].trim();
                self.pos += end_idx + 1;
                let (name, _rest) = content.split_once(char::is_whitespace).unwrap_or((content, ""));
                return Ok(Some(XmlPullEvent::DocType {
                    name: name.trim(),
                    public_id: None,
                    system_id: None,
                }));
            }
        }

        if remaining.starts_with("</") {
            if let Some(end_idx) = remaining.find('>') {
                let tag_name = remaining[2..end_idx].trim();
                self.pos += end_idx + 1;
                return Ok(Some(XmlPullEvent::EndElement { name: tag_name }));
            }
        }

        if remaining.starts_with('<') {
            if let Some(end_idx) = remaining.find('>') {
                let is_self_closing = remaining[..end_idx].ends_with('/');
                let raw_tag = if is_self_closing {
                    &remaining[1..end_idx - 1]
                } else {
                    &remaining[1..end_idx]
                };

                self.pos += end_idx + 1;
                let (tag_name, attr_raw) = raw_tag
                    .split_once(char::is_whitespace)
                    .unwrap_or((raw_tag, ""));

                let clean_name = tag_name.trim();
                if is_self_closing {
                    self.pending_end_element = Some(clean_name);
                }

                return Ok(Some(XmlPullEvent::StartElement {
                    name: clean_name,
                    attr_raw: attr_raw.trim(),
                }));
            }
        }

        if remaining.starts_with('<') {
            return Err(crate::error::XmlError::SyntaxError {
                message: "Unclosed XML markup or tag at EOF".into(),
                line: 1,
                col: 1,
            });
        }

        // Text content up to next '<'
        let next_tag = remaining.find('<').unwrap_or(remaining.len());
        if next_tag == 0 {
            return Ok(None);
        }
        let text = &remaining[..next_tag];
        self.pos += next_tag;
        Ok(Some(XmlPullEvent::Text(text)))
    }
}

impl<'a> Iterator for XmlPullParser<'a> {
    type Item = Result<XmlPullEvent<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_event() {
            Ok(Some(XmlPullEvent::EndDocument)) => None,
            Ok(Some(ev)) => Some(Ok(ev)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}
