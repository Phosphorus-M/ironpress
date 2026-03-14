use crate::parser::css::{CssValue, StyleMap};
use crate::parser::dom::HtmlTag;
use crate::style::defaults::default_style;
use crate::types::{Color, EdgeSizes};

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Font weight.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontWeight {
    #[default]
    Normal,
    Bold,
}

/// Font style.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}

/// Fully resolved style for a node.
#[derive(Debug, Clone)]
pub struct ComputedStyle {
    pub font_size: f32,
    pub font_weight: FontWeight,
    pub font_style: FontStyle,
    pub color: Color,
    pub background_color: Option<Color>,
    pub margin: EdgeSizes,
    pub padding: EdgeSizes,
    pub text_align: TextAlign,
    pub text_decoration_underline: bool,
    pub line_height: f32,
    pub page_break_before: bool,
    pub page_break_after: bool,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            font_weight: FontWeight::Normal,
            font_style: FontStyle::Normal,
            color: Color::BLACK,
            background_color: None,
            margin: EdgeSizes::default(),
            padding: EdgeSizes::default(),
            text_align: TextAlign::Left,
            text_decoration_underline: false,
            line_height: 1.4,
            page_break_before: false,
            page_break_after: false,
        }
    }
}

/// Compute the style for a node given its tag, inline styles, and parent style.
pub fn compute_style(
    tag: HtmlTag,
    inline_style: Option<&str>,
    parent: &ComputedStyle,
) -> ComputedStyle {
    let mut style = parent.clone();

    // Reset block-level properties that don't inherit
    if tag.is_block() {
        style.margin = EdgeSizes::default();
        style.padding = EdgeSizes::default();
        style.background_color = None;
    }

    // Apply tag defaults
    let defaults = default_style(tag);
    apply_style_map(&mut style, &defaults);

    // Apply inline styles (override defaults)
    if let Some(css_str) = inline_style {
        let inline = crate::parser::css::parse_inline_style(css_str);
        apply_style_map(&mut style, &inline);
    }

    style
}

fn apply_style_map(style: &mut ComputedStyle, map: &StyleMap) {
    if let Some(CssValue::Length(v)) = map.get("font-size") {
        style.font_size = *v;
    }
    if let Some(CssValue::Number(v)) = map.get("font-size") {
        // em value — multiply by current font-size
        style.font_size *= *v;
    }

    if let Some(CssValue::Keyword(k)) = map.get("font-weight") {
        style.font_weight = if k == "bold" || k == "700" || k == "800" || k == "900" {
            FontWeight::Bold
        } else {
            FontWeight::Normal
        };
    }

    if let Some(CssValue::Keyword(k)) = map.get("font-style") {
        style.font_style = if k == "italic" || k == "oblique" {
            FontStyle::Italic
        } else {
            FontStyle::Normal
        };
    }

    if let Some(CssValue::Color(c)) = map.get("color") {
        style.color = *c;
    }

    if let Some(CssValue::Color(c)) = map.get("background-color") {
        style.background_color = Some(*c);
    }

    if let Some(CssValue::Length(v)) = map.get("margin-top") {
        style.margin.top = *v;
    }
    if let Some(CssValue::Length(v)) = map.get("margin-right") {
        style.margin.right = *v;
    }
    if let Some(CssValue::Length(v)) = map.get("margin-bottom") {
        style.margin.bottom = *v;
    }
    if let Some(CssValue::Length(v)) = map.get("margin-left") {
        style.margin.left = *v;
    }

    if let Some(CssValue::Length(v)) = map.get("padding-top") {
        style.padding.top = *v;
    }
    if let Some(CssValue::Length(v)) = map.get("padding-right") {
        style.padding.right = *v;
    }
    if let Some(CssValue::Length(v)) = map.get("padding-bottom") {
        style.padding.bottom = *v;
    }
    if let Some(CssValue::Length(v)) = map.get("padding-left") {
        style.padding.left = *v;
    }

    if let Some(CssValue::Keyword(k)) = map.get("text-align") {
        style.text_align = match k.as_str() {
            "center" => TextAlign::Center,
            "right" => TextAlign::Right,
            _ => TextAlign::Left,
        };
    }

    if let Some(CssValue::Keyword(k)) = map.get("text-decoration") {
        style.text_decoration_underline = k == "underline";
    }

    if let Some(CssValue::Number(v)) = map.get("line-height") {
        style.line_height = *v;
    }
    if let Some(CssValue::Length(v)) = map.get("line-height") {
        style.line_height = *v / style.font_size;
    }

    if let Some(CssValue::Keyword(k)) = map.get("page-break-before") {
        style.page_break_before = k == "always";
    }
    if let Some(CssValue::Keyword(k)) = map.get("page-break-after") {
        style.page_break_after = k == "always";
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn h1_defaults() {
        let parent = ComputedStyle::default();
        let style = compute_style(HtmlTag::H1, None, &parent);
        assert_eq!(style.font_size, 24.0);
        assert_eq!(style.font_weight, FontWeight::Bold);
    }

    #[test]
    fn inline_overrides_defaults() {
        let parent = ComputedStyle::default();
        let style = compute_style(HtmlTag::H1, Some("font-size: 36pt"), &parent);
        assert_eq!(style.font_size, 36.0);
        assert_eq!(style.font_weight, FontWeight::Bold); // still bold from defaults
    }

    #[test]
    fn color_inherited() {
        let mut parent = ComputedStyle::default();
        parent.color = Color::rgb(255, 0, 0);
        let style = compute_style(HtmlTag::Span, None, &parent);
        assert_eq!(style.color.r, 255);
    }

    #[test]
    fn bold_tag() {
        let parent = ComputedStyle::default();
        let style = compute_style(HtmlTag::Strong, None, &parent);
        assert_eq!(style.font_weight, FontWeight::Bold);
    }

    #[test]
    fn italic_tag() {
        let parent = ComputedStyle::default();
        let style = compute_style(HtmlTag::Em, None, &parent);
        assert_eq!(style.font_style, FontStyle::Italic);
    }
}
