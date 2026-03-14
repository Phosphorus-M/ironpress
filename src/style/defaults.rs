use crate::parser::css::{CssValue, StyleMap};
use crate::parser::dom::HtmlTag;
use crate::types::Color;

/// Returns the default (user-agent) styles for a given HTML tag.
pub fn default_style(tag: HtmlTag) -> StyleMap {
    let mut style = StyleMap::new();

    match tag {
        HtmlTag::H1 => {
            style.set("font-size", CssValue::Length(24.0));
            style.set("font-weight", CssValue::Keyword("bold".into()));
            style.set("margin-top", CssValue::Length(12.0));
            style.set("margin-bottom", CssValue::Length(12.0));
        }
        HtmlTag::H2 => {
            style.set("font-size", CssValue::Length(20.0));
            style.set("font-weight", CssValue::Keyword("bold".into()));
            style.set("margin-top", CssValue::Length(10.0));
            style.set("margin-bottom", CssValue::Length(10.0));
        }
        HtmlTag::H3 => {
            style.set("font-size", CssValue::Length(16.0));
            style.set("font-weight", CssValue::Keyword("bold".into()));
            style.set("margin-top", CssValue::Length(8.0));
            style.set("margin-bottom", CssValue::Length(8.0));
        }
        HtmlTag::H4 => {
            style.set("font-size", CssValue::Length(14.0));
            style.set("font-weight", CssValue::Keyword("bold".into()));
            style.set("margin-top", CssValue::Length(6.0));
            style.set("margin-bottom", CssValue::Length(6.0));
        }
        HtmlTag::H5 => {
            style.set("font-size", CssValue::Length(12.0));
            style.set("font-weight", CssValue::Keyword("bold".into()));
            style.set("margin-top", CssValue::Length(4.0));
            style.set("margin-bottom", CssValue::Length(4.0));
        }
        HtmlTag::H6 => {
            style.set("font-size", CssValue::Length(10.0));
            style.set("font-weight", CssValue::Keyword("bold".into()));
            style.set("margin-top", CssValue::Length(4.0));
            style.set("margin-bottom", CssValue::Length(4.0));
        }
        HtmlTag::P => {
            style.set("font-size", CssValue::Length(12.0));
            style.set("margin-top", CssValue::Length(0.0));
            style.set("margin-bottom", CssValue::Length(8.0));
        }
        HtmlTag::Strong | HtmlTag::B => {
            style.set("font-weight", CssValue::Keyword("bold".into()));
        }
        HtmlTag::Em | HtmlTag::I => {
            style.set("font-style", CssValue::Keyword("italic".into()));
        }
        HtmlTag::U => {
            style.set("text-decoration", CssValue::Keyword("underline".into()));
        }
        HtmlTag::A => {
            style.set("color", CssValue::Color(Color::rgb(0, 0, 238)));
            style.set("text-decoration", CssValue::Keyword("underline".into()));
        }
        HtmlTag::Hr => {
            style.set("margin-top", CssValue::Length(6.0));
            style.set("margin-bottom", CssValue::Length(6.0));
        }
        HtmlTag::Li => {
            style.set("margin-bottom", CssValue::Length(2.0));
        }
        HtmlTag::Ul | HtmlTag::Ol => {
            style.set("margin-top", CssValue::Length(4.0));
            style.set("margin-bottom", CssValue::Length(8.0));
            style.set("margin-left", CssValue::Length(20.0));
        }
        HtmlTag::Td => {
            style.set("padding-top", CssValue::Length(4.0));
            style.set("padding-right", CssValue::Length(6.0));
            style.set("padding-bottom", CssValue::Length(4.0));
            style.set("padding-left", CssValue::Length(6.0));
        }
        HtmlTag::Th => {
            style.set("padding-top", CssValue::Length(4.0));
            style.set("padding-right", CssValue::Length(6.0));
            style.set("padding-bottom", CssValue::Length(4.0));
            style.set("padding-left", CssValue::Length(6.0));
            style.set("font-weight", CssValue::Keyword("bold".into()));
        }
        _ => {}
    }

    style
}
