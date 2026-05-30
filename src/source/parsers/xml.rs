use quick_xml::events::BytesText;
use quick_xml::{XmlVersion, escape::unescape};
use std::borrow::Cow;

pub(super) fn decode_xml_text(text: &BytesText<'_>) -> String {
    let decoded = text
        .decode()
        .unwrap_or_else(|_| Cow::Owned(String::from_utf8_lossy(text.as_ref()).into_owned()));

    unescape(&decoded)
        .map(Cow::into_owned)
        .unwrap_or_else(|_| decoded.into_owned())
}

pub(super) fn xml_attr_value<'a>(
    attr: &quick_xml::events::attributes::Attribute<'a>,
) -> quick_xml::Result<Cow<'a, str>> {
    attr.normalized_value(XmlVersion::Implicit1_0)
}
