use crate::parser::dom::{DomNode, ElementNode, HtmlTag};
use crate::style::computed::{compute_style, ComputedStyle, FontStyle, FontWeight, TextAlign};
use crate::types::{Margin, PageSize};

/// A styled text run (a piece of text with uniform style).
#[derive(Debug, Clone)]
pub struct TextRun {
    pub text: String,
    pub font_size: f32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: (f32, f32, f32),
}

/// A laid-out line of text runs.
#[derive(Debug, Clone)]
pub struct TextLine {
    pub runs: Vec<TextRun>,
    pub height: f32,
}

/// A layout element ready for rendering.
#[derive(Debug)]
pub enum LayoutElement {
    /// A block of text lines with optional background.
    TextBlock {
        lines: Vec<TextLine>,
        margin_top: f32,
        margin_bottom: f32,
        text_align: TextAlign,
        background_color: Option<(f32, f32, f32)>,
        padding_top: f32,
        padding_bottom: f32,
        padding_left: f32,
        padding_right: f32,
    },
    /// A horizontal rule.
    HorizontalRule { margin_top: f32, margin_bottom: f32 },
    /// A page break.
    PageBreak,
}

/// A fully laid-out page.
pub struct Page {
    pub elements: Vec<(f32, LayoutElement)>, // (y_position, element)
}

/// Lay out the DOM nodes into pages.
pub fn layout(nodes: &[DomNode], page_size: PageSize, margin: Margin) -> Vec<Page> {
    let parent_style = ComputedStyle::default();
    let available_width = page_size.width - margin.left - margin.right;
    let content_height = page_size.height - margin.top - margin.bottom;

    // First, flatten DOM into layout elements
    let mut elements = Vec::new();
    flatten_nodes(nodes, &parent_style, available_width, &mut elements);

    // Then paginate
    paginate(elements, content_height)
}

fn flatten_nodes(
    nodes: &[DomNode],
    parent_style: &ComputedStyle,
    available_width: f32,
    output: &mut Vec<LayoutElement>,
) {
    for node in nodes {
        match node {
            DomNode::Text(text) => {
                let trimmed = collapse_whitespace(text);
                if !trimmed.is_empty() {
                    let run = TextRun {
                        text: trimmed,
                        font_size: parent_style.font_size,
                        bold: parent_style.font_weight == FontWeight::Bold,
                        italic: parent_style.font_style == FontStyle::Italic,
                        underline: parent_style.text_decoration_underline,
                        color: parent_style.color.to_f32_rgb(),
                    };
                    let _line_height = parent_style.font_size * parent_style.line_height;
                    let lines = wrap_text_runs(vec![run], available_width, parent_style.font_size);
                    if !lines.is_empty() {
                        output.push(LayoutElement::TextBlock {
                            lines,
                            margin_top: 0.0,
                            margin_bottom: 0.0,
                            text_align: parent_style.text_align,
                            background_color: None,
                            padding_top: 0.0,
                            padding_bottom: 0.0,
                            padding_left: 0.0,
                            padding_right: 0.0,
                        });
                    }
                }
            }
            DomNode::Element(el) => {
                flatten_element(el, parent_style, available_width, output);
            }
        }
    }
}

fn flatten_element(
    el: &ElementNode,
    parent_style: &ComputedStyle,
    available_width: f32,
    output: &mut Vec<LayoutElement>,
) {
    let style = compute_style(el.tag, el.style_attr(), parent_style);

    if el.tag == HtmlTag::Br {
        let line = TextLine {
            runs: vec![TextRun {
                text: String::new(),
                font_size: style.font_size,
                bold: false,
                italic: false,
                underline: false,
                color: (0.0, 0.0, 0.0),
            }],
            height: style.font_size * style.line_height,
        };
        output.push(LayoutElement::TextBlock {
            lines: vec![line],
            margin_top: 0.0,
            margin_bottom: 0.0,
            text_align: TextAlign::Left,
            background_color: None,
            padding_top: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            padding_right: 0.0,
        });
        return;
    }

    if el.tag == HtmlTag::Hr {
        output.push(LayoutElement::HorizontalRule {
            margin_top: style.margin.top,
            margin_bottom: style.margin.bottom,
        });
        return;
    }

    if style.page_break_before {
        output.push(LayoutElement::PageBreak);
    }

    if el.tag.is_block() {
        // Collect all inline content as text runs
        let inner_width = available_width - style.padding.left - style.padding.right;
        let mut runs = Vec::new();
        collect_text_runs(&el.children, &style, &mut runs);

        if !runs.is_empty() {
            let lines = wrap_text_runs(runs, inner_width, style.font_size);
            let bg = style
                .background_color
                .map(|c: crate::types::Color| c.to_f32_rgb());

            output.push(LayoutElement::TextBlock {
                lines,
                margin_top: style.margin.top,
                margin_bottom: style.margin.bottom,
                text_align: style.text_align,
                background_color: bg,
                padding_top: style.padding.top,
                padding_bottom: style.padding.bottom,
                padding_left: style.padding.left,
                padding_right: style.padding.right,
            });
        }

        // Also process block children recursively
        for child in &el.children {
            if let DomNode::Element(child_el) = child {
                if child_el.tag.is_block() {
                    flatten_element(child_el, &style, available_width, output);
                }
            }
        }
    } else {
        // Inline element — process children with this style context
        flatten_nodes(&el.children, &style, available_width, output);
    }

    if style.page_break_after {
        output.push(LayoutElement::PageBreak);
    }
}

fn collect_text_runs(nodes: &[DomNode], parent_style: &ComputedStyle, runs: &mut Vec<TextRun>) {
    for node in nodes {
        match node {
            DomNode::Text(text) => {
                let trimmed = collapse_whitespace(text);
                if !trimmed.is_empty() {
                    runs.push(TextRun {
                        text: trimmed,
                        font_size: parent_style.font_size,
                        bold: parent_style.font_weight == FontWeight::Bold,
                        italic: parent_style.font_style == FontStyle::Italic,
                        underline: parent_style.text_decoration_underline,
                        color: parent_style.color.to_f32_rgb(),
                    });
                }
            }
            DomNode::Element(el) => {
                if el.tag.is_inline() || el.tag == HtmlTag::Br {
                    if el.tag == HtmlTag::Br {
                        runs.push(TextRun {
                            text: "\n".to_string(),
                            font_size: parent_style.font_size,
                            bold: false,
                            italic: false,
                            underline: false,
                            color: (0.0, 0.0, 0.0),
                        });
                    } else {
                        let style = compute_style(el.tag, el.style_attr(), parent_style);
                        collect_text_runs(&el.children, &style, runs);
                    }
                }
            }
        }
    }
}

/// Simple text wrapping using character width estimation.
/// Uses 0.5 * font_size as average character width for the built-in font.
fn wrap_text_runs(runs: Vec<TextRun>, max_width: f32, default_font_size: f32) -> Vec<TextLine> {
    let mut lines: Vec<TextLine> = Vec::new();
    let mut current_runs: Vec<TextRun> = Vec::new();
    let mut current_width: f32 = 0.0;
    let mut line_height = default_font_size * 1.4;

    // Concatenate all text then re-split by words, preserving run styles
    let mut styled_words: Vec<(String, TextRun)> = Vec::new();
    for run in &runs {
        if run.text == "\n" {
            styled_words.push(("\n".to_string(), run.clone()));
            continue;
        }
        for word in run.text.split_whitespace() {
            styled_words.push((word.to_string(), run.clone()));
        }
    }

    for (word, template) in styled_words {
        if word == "\n" {
            // Line break
            lines.push(TextLine {
                runs: std::mem::take(&mut current_runs),
                height: line_height,
            });
            current_width = 0.0;
            line_height = default_font_size * 1.4;
            continue;
        }

        let char_width = template.font_size * 0.5;
        let word_width = word.len() as f32 * char_width;
        let space_width = char_width;

        let needed = if current_width > 0.0 {
            space_width + word_width
        } else {
            word_width
        };

        if current_width + needed > max_width && current_width > 0.0 {
            // Wrap to new line
            lines.push(TextLine {
                runs: std::mem::take(&mut current_runs),
                height: line_height,
            });
            current_width = 0.0;
            line_height = default_font_size * 1.4;
        }

        let text = if current_width > 0.0 {
            format!(" {word}")
        } else {
            word
        };

        let w = text.len() as f32 * template.font_size * 0.5;
        current_width += w;
        line_height = line_height.max(template.font_size * 1.4);

        current_runs.push(TextRun { text, ..template });
    }

    if !current_runs.is_empty() {
        lines.push(TextLine {
            runs: current_runs,
            height: line_height,
        });
    }

    lines
}

fn paginate(elements: Vec<LayoutElement>, content_height: f32) -> Vec<Page> {
    let mut pages: Vec<Page> = Vec::new();
    let mut current_elements: Vec<(f32, LayoutElement)> = Vec::new();
    let mut y = 0.0;

    for element in elements {
        let (element_height, margin_top_val) = match &element {
            LayoutElement::PageBreak => {
                pages.push(Page {
                    elements: std::mem::take(&mut current_elements),
                });
                y = 0.0;
                continue;
            }
            LayoutElement::HorizontalRule {
                margin_top,
                margin_bottom,
            } => (*margin_top + 1.0 + *margin_bottom, *margin_top),
            LayoutElement::TextBlock {
                lines,
                margin_top,
                margin_bottom,
                padding_top,
                padding_bottom,
                ..
            } => {
                let text_height: f32 = lines.iter().map(|l| l.height).sum();
                let total = margin_top + padding_top + text_height + padding_bottom + margin_bottom;
                (total, *margin_top)
            }
        };

        if y + element_height > content_height && y > 0.0 {
            pages.push(Page {
                elements: std::mem::take(&mut current_elements),
            });
            y = 0.0;
        }

        y += margin_top_val;
        let after_margin = element_height - margin_top_val;
        current_elements.push((y, element));
        y += after_margin;
    }

    if !current_elements.is_empty() {
        pages.push(Page {
            elements: current_elements,
        });
    }

    if pages.is_empty() {
        pages.push(Page {
            elements: Vec::new(),
        });
    }

    pages
}

fn collapse_whitespace(text: &str) -> String {
    let mut result = String::new();
    let mut last_was_space = false;
    for c in text.chars() {
        if c.is_whitespace() {
            if !last_was_space && !result.is_empty() {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(c);
            last_was_space = false;
        }
    }
    result.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::html::parse_html;

    #[test]
    fn layout_simple_paragraph() {
        let nodes = parse_html("<p>Hello World</p>").unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        assert!(!pages[0].elements.is_empty());
    }

    #[test]
    fn layout_multiple_elements() {
        let nodes = parse_html("<h1>Title</h1><p>Paragraph one.</p><p>Paragraph two.</p>").unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        assert!(pages[0].elements.len() >= 3);
    }

    #[test]
    fn layout_empty() {
        let nodes = parse_html("").unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        assert!(pages[0].elements.is_empty());
    }

    #[test]
    fn collapse_whitespace_test() {
        assert_eq!(collapse_whitespace("  hello   world  "), "hello world");
        assert_eq!(collapse_whitespace("\n\t  foo  \n"), "foo");
    }

    #[test]
    fn page_break_creates_new_page() {
        let html = r#"<p>Page 1</p><div style="page-break-before: always"><p>Page 2</p></div>"#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert!(pages.len() >= 2);
    }

    #[test]
    fn bare_text_node() {
        // Text not wrapped in any element — exercises DomNode::Text branch in flatten_nodes
        let nodes = parse_html("Just some bare text").unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        assert!(!pages[0].elements.is_empty());
    }

    #[test]
    fn br_element_creates_empty_line() {
        let html = "<p>Line one</p><br><p>Line two</p>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        // Should have at least 3 elements (p, br, p)
        assert!(pages[0].elements.len() >= 2);
    }

    #[test]
    fn inline_element_layout() {
        // Inline element outside a block — exercises the else branch
        let html = "<span>Hello</span>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
    }

    #[test]
    fn page_break_after() {
        let html = r#"<div style="page-break-after: always"><p>Page 1</p></div><p>Page 2</p>"#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert!(pages.len() >= 2);
    }

    #[test]
    fn word_wrap_long_text() {
        // Generate text that exceeds page width to trigger word wrapping
        let long_text = "word ".repeat(200);
        let html = format!("<p>{long_text}</p>");
        let nodes = parse_html(&html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        // Should have wrapped into multiple lines
        if let (_, LayoutElement::TextBlock { lines, .. }) = &pages[0].elements[0] {
            assert!(lines.len() > 1);
        }
    }

    #[test]
    fn content_overflows_to_next_page() {
        // Generate enough content to overflow one page
        let paragraphs = "<p>Some paragraph text that takes up space.</p>\n".repeat(100);
        let nodes = parse_html(&paragraphs).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert!(pages.len() >= 2);
    }

    #[test]
    fn background_color_block() {
        let html = r#"<div style="background-color: yellow"><p>Highlighted</p></div>"#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert!(!pages[0].elements.is_empty());
    }

    #[test]
    fn pre_element_with_background() {
        let html = "<pre>code block</pre>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        // Pre has background color in defaults
        if let (
            _,
            LayoutElement::TextBlock {
                background_color, ..
            },
        ) = &pages[0].elements[0]
        {
            assert!(background_color.is_some());
        }
    }
}
