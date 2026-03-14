use std::collections::HashMap;

/// Supported HTML tags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlTag {
    Html,
    Head,
    Body,
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
    P,
    Div,
    Span,
    Strong,
    B,
    Em,
    I,
    U,
    A,
    Br,
    Hr,
    Table,
    Tr,
    Td,
    Th,
    Ul,
    Ol,
    Li,
    Img,
    Unknown,
}

impl HtmlTag {
    pub fn from_tag_name(tag: &str) -> Self {
        match tag.to_ascii_lowercase().as_str() {
            "html" => Self::Html,
            "head" => Self::Head,
            "body" => Self::Body,
            "h1" => Self::H1,
            "h2" => Self::H2,
            "h3" => Self::H3,
            "h4" => Self::H4,
            "h5" => Self::H5,
            "h6" => Self::H6,
            "p" => Self::P,
            "div" => Self::Div,
            "span" => Self::Span,
            "strong" => Self::Strong,
            "b" => Self::B,
            "em" => Self::Em,
            "i" => Self::I,
            "u" => Self::U,
            "a" => Self::A,
            "br" => Self::Br,
            "hr" => Self::Hr,
            "table" => Self::Table,
            "tr" => Self::Tr,
            "td" => Self::Td,
            "th" => Self::Th,
            "ul" => Self::Ul,
            "ol" => Self::Ol,
            "li" => Self::Li,
            "img" => Self::Img,
            _ => Self::Unknown,
        }
    }

    pub fn is_block(&self) -> bool {
        matches!(
            self,
            Self::H1
                | Self::H2
                | Self::H3
                | Self::H4
                | Self::H5
                | Self::H6
                | Self::P
                | Self::Div
                | Self::Table
                | Self::Tr
                | Self::Ul
                | Self::Ol
                | Self::Li
                | Self::Hr
                | Self::Body
                | Self::Html
        )
    }

    pub fn is_inline(&self) -> bool {
        matches!(
            self,
            Self::Span | Self::Strong | Self::B | Self::Em | Self::I | Self::U | Self::A
        )
    }
}

/// A node in the internal DOM tree.
#[derive(Debug)]
pub enum DomNode {
    Element(ElementNode),
    Text(String),
}

/// An HTML element with tag, attributes, and children.
#[derive(Debug)]
pub struct ElementNode {
    pub tag: HtmlTag,
    pub attributes: HashMap<String, String>,
    pub children: Vec<DomNode>,
}

impl ElementNode {
    pub fn new(tag: HtmlTag) -> Self {
        Self {
            tag,
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    pub fn style_attr(&self) -> Option<&str> {
        self.attributes.get("style").map(|s| s.as_str())
    }
}
