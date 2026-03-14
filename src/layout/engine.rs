use crate::parser::css::CssRule;
use crate::parser::dom::{DomNode, ElementNode, HtmlTag};
use crate::style::computed::{
    ComputedStyle, FontFamily, FontStyle, FontWeight, TextAlign, compute_style,
    compute_style_with_rules,
};
use crate::types::{Margin, PageSize};

/// Context for rendering list items.
#[derive(Debug, Clone)]
enum ListContext {
    Unordered { indent: f32 },
    Ordered { index: usize, indent: f32 },
}

/// A table cell ready for rendering.
#[derive(Debug)]
pub struct TableCell {
    pub lines: Vec<TextLine>,
    pub bold: bool,
    pub background_color: Option<(f32, f32, f32)>,
    pub padding_top: f32,
    pub padding_right: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    /// Number of columns this cell spans (default 1).
    pub colspan: usize,
    /// Number of rows this cell spans (default 1).
    pub rowspan: usize,
}

/// A styled text run (a piece of text with uniform style).
#[derive(Debug, Clone)]
pub struct TextRun {
    pub text: String,
    pub font_size: f32,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub line_through: bool,
    pub color: (f32, f32, f32),
    pub link_url: Option<String>,
    pub font_family: FontFamily,
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
        border_width: f32,
        border_color: Option<(f32, f32, f32)>,
    },
    /// A table row with cells.
    TableRow {
        cells: Vec<TableCell>,
        col_width: f32,
        margin_top: f32,
        margin_bottom: f32,
    },
    /// An embedded image (JPEG).
    Image {
        data: Vec<u8>,
        width: f32,
        height: f32,
        margin_top: f32,
        margin_bottom: f32,
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
    layout_with_rules(nodes, page_size, margin, &[])
}

/// Lay out the DOM nodes into pages with stylesheet rules.
pub fn layout_with_rules(
    nodes: &[DomNode],
    page_size: PageSize,
    margin: Margin,
    rules: &[CssRule],
) -> Vec<Page> {
    let parent_style = ComputedStyle::default();
    let available_width = page_size.width - margin.left - margin.right;
    let content_height = page_size.height - margin.top - margin.bottom;

    // First, flatten DOM into layout elements
    let mut elements = Vec::new();
    flatten_nodes(
        nodes,
        &parent_style,
        available_width,
        &mut elements,
        None,
        rules,
    );

    // Then paginate
    paginate(elements, content_height)
}

fn flatten_nodes(
    nodes: &[DomNode],
    parent_style: &ComputedStyle,
    available_width: f32,
    output: &mut Vec<LayoutElement>,
    list_ctx: Option<&ListContext>,
    rules: &[CssRule],
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
                        line_through: parent_style.text_decoration_line_through,
                        color: parent_style.color.to_f32_rgb(),
                        link_url: None,
                        font_family: parent_style.font_family,
                    };
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
                            border_width: 0.0,
                            border_color: None,
                        });
                    }
                }
            }
            DomNode::Element(el) => {
                flatten_element(el, parent_style, available_width, output, list_ctx, rules);
            }
        }
    }
}

fn flatten_element(
    el: &ElementNode,
    parent_style: &ComputedStyle,
    available_width: f32,
    output: &mut Vec<LayoutElement>,
    list_ctx: Option<&ListContext>,
    rules: &[CssRule],
) {
    let classes = el.class_list();
    let style = compute_style_with_rules(
        el.tag,
        el.style_attr(),
        parent_style,
        rules,
        el.tag_name(),
        &classes,
        el.id(),
    );

    if el.tag == HtmlTag::Br {
        let line = TextLine {
            runs: vec![TextRun {
                text: String::new(),
                font_size: style.font_size,
                bold: false,
                italic: false,
                underline: false,
                line_through: false,
                color: (0.0, 0.0, 0.0),
                link_url: None,
                font_family: style.font_family,
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
            border_width: 0.0,
            border_color: None,
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

    if el.tag == HtmlTag::Img {
        if let Some(image_data) = load_image_data(el) {
            let (mut w, mut h) = parse_image_dimensions(el);
            // Scale to fit within available width
            if w > available_width {
                let scale = available_width / w;
                w = available_width;
                h *= scale;
            }
            output.push(LayoutElement::Image {
                data: image_data,
                width: w,
                height: h,
                margin_top: style.margin.top,
                margin_bottom: style.margin.bottom,
            });
        }
        return;
    }

    if style.page_break_before {
        output.push(LayoutElement::PageBreak);
    }

    // Table handling
    if el.tag == HtmlTag::Table {
        flatten_table(el, &style, available_width, output, rules);
        return;
    }

    // List handling — Ul/Ol pass context to Li children
    if el.tag == HtmlTag::Ul || el.tag == HtmlTag::Ol {
        let inner_width = available_width - style.margin.left;
        // Accumulate indentation from parent list context
        let parent_indent = match list_ctx {
            Some(ListContext::Unordered { indent }) => *indent,
            Some(ListContext::Ordered { indent, .. }) => *indent,
            None => 0.0,
        };
        let total_indent = parent_indent + style.margin.left;
        let mut ctx = if el.tag == HtmlTag::Ol {
            ListContext::Ordered {
                index: 1,
                indent: total_indent,
            }
        } else {
            ListContext::Unordered {
                indent: total_indent,
            }
        };
        for child in &el.children {
            if let DomNode::Element(child_el) = child {
                if child_el.tag == HtmlTag::Li {
                    flatten_element(child_el, &style, inner_width, output, Some(&ctx), rules);
                    if let ListContext::Ordered { index, .. } = &mut ctx {
                        *index += 1;
                    }
                } else {
                    flatten_element(child_el, &style, inner_width, output, None, rules);
                }
            }
        }
        return;
    }

    // Li handling — prepend bullet/number marker
    if el.tag == HtmlTag::Li {
        let inner_width = available_width - style.padding.left - style.padding.right;
        let mut runs = Vec::new();

        // Add list marker
        let marker = match list_ctx {
            Some(ListContext::Unordered { .. }) => "- ".to_string(),
            Some(ListContext::Ordered { index, .. }) => format!("{index}. "),
            None => "- ".to_string(),
        };
        let list_indent = match list_ctx {
            Some(ListContext::Unordered { indent }) => *indent,
            Some(ListContext::Ordered { indent, .. }) => *indent,
            None => 0.0,
        };
        runs.push(TextRun {
            text: marker,
            font_size: style.font_size,
            bold: style.font_weight == FontWeight::Bold,
            italic: style.font_style == FontStyle::Italic,
            underline: false,
            line_through: false,
            color: style.color.to_f32_rgb(),
            link_url: None,
            font_family: style.font_family,
        });

        collect_text_runs(&el.children, &style, &mut runs, None);

        if !runs.is_empty() {
            let lines = wrap_text_runs(runs, inner_width, style.font_size);
            output.push(LayoutElement::TextBlock {
                lines,
                margin_top: style.margin.top,
                margin_bottom: style.margin.bottom,
                text_align: style.text_align,
                background_color: None,
                padding_top: 0.0,
                padding_bottom: 0.0,
                padding_left: list_indent,
                padding_right: 0.0,
                border_width: 0.0,
                border_color: None,
            });
        }

        // Process block children inside li (nested lists get reduced width for indentation)
        for child in &el.children {
            if let DomNode::Element(child_el) = child {
                if child_el.tag == HtmlTag::Ul || child_el.tag == HtmlTag::Ol {
                    flatten_element(child_el, &style, inner_width, output, list_ctx, rules);
                } else if child_el.tag.is_block() {
                    flatten_element(child_el, &style, available_width, output, None, rules);
                }
            }
        }
        return;
    }

    if el.tag.is_block() {
        // Collect all inline content as text runs
        let inner_width = available_width - style.padding.left - style.padding.right;
        let mut runs = Vec::new();
        collect_text_runs(&el.children, &style, &mut runs, None);

        if !runs.is_empty() {
            let lines = wrap_text_runs(runs, inner_width, style.font_size);
            let bg = style
                .background_color
                .map(|c: crate::types::Color| c.to_f32_rgb());

            let border_clr = style.border_color.map(|c| c.to_f32_rgb());
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
                border_width: style.border_width,
                border_color: border_clr,
            });
        }

        // Also process block children recursively
        for child in &el.children {
            if let DomNode::Element(child_el) = child {
                if child_el.tag.is_block() {
                    flatten_element(child_el, &style, available_width, output, None, rules);
                }
            }
        }
    } else {
        // Inline element — process children with this style context
        flatten_nodes(&el.children, &style, available_width, output, None, rules);
    }

    if style.page_break_after {
        output.push(LayoutElement::PageBreak);
    }
}

fn flatten_table(
    el: &ElementNode,
    style: &ComputedStyle,
    available_width: f32,
    output: &mut Vec<LayoutElement>,
    _rules: &[CssRule],
) {
    let inner_width = available_width - style.margin.left - style.margin.right;

    // Collect all <tr> elements (from direct children, thead, tbody, tfoot)
    let mut rows: Vec<&ElementNode> = Vec::new();
    for child in &el.children {
        if let DomNode::Element(child_el) = child {
            match child_el.tag {
                HtmlTag::Tr => rows.push(child_el),
                HtmlTag::Thead | HtmlTag::Tbody | HtmlTag::Tfoot => {
                    for grandchild in &child_el.children {
                        if let DomNode::Element(gc) = grandchild {
                            if gc.tag == HtmlTag::Tr {
                                rows.push(gc);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if rows.is_empty() {
        return;
    }

    // Determine column count from the widest row, accounting for colspan
    let num_cols = rows
        .iter()
        .map(|row| {
            row.children
                .iter()
                .filter_map(|c| {
                    if let DomNode::Element(e) = c {
                        if e.tag == HtmlTag::Td || e.tag == HtmlTag::Th {
                            let colspan = e
                                .attributes
                                .get("colspan")
                                .and_then(|v| v.parse::<usize>().ok())
                                .unwrap_or(1)
                                .max(1);
                            return Some(colspan);
                        }
                    }
                    None
                })
                .sum::<usize>()
        })
        .max()
        .unwrap_or(1);

    let col_width = inner_width / num_cols as f32;

    // Build layout rows
    let mut is_first = true;
    for row in &rows {
        let row_style = compute_style(row.tag, row.style_attr(), style);
        let mut cells = Vec::new();

        for child in &row.children {
            if let DomNode::Element(cell_el) = child {
                if cell_el.tag == HtmlTag::Td || cell_el.tag == HtmlTag::Th {
                    let colspan = cell_el
                        .attributes
                        .get("colspan")
                        .and_then(|v| v.parse::<usize>().ok())
                        .unwrap_or(1)
                        .max(1);
                    let rowspan = cell_el
                        .attributes
                        .get("rowspan")
                        .and_then(|v| v.parse::<usize>().ok())
                        .unwrap_or(1)
                        .max(1);

                    let cell_style = compute_style(cell_el.tag, cell_el.style_attr(), &row_style);
                    let effective_width = col_width * colspan as f32;
                    let cell_inner =
                        effective_width - cell_style.padding.left - cell_style.padding.right;

                    let mut runs = Vec::new();
                    collect_text_runs(&cell_el.children, &cell_style, &mut runs, None);
                    let lines = wrap_text_runs(runs, cell_inner.max(1.0), cell_style.font_size);

                    let bg = cell_style
                        .background_color
                        .map(|c: crate::types::Color| c.to_f32_rgb());

                    cells.push(TableCell {
                        lines,
                        bold: cell_style.font_weight == FontWeight::Bold,
                        background_color: bg,
                        padding_top: cell_style.padding.top,
                        padding_right: cell_style.padding.right,
                        padding_bottom: cell_style.padding.bottom,
                        padding_left: cell_style.padding.left,
                        colspan,
                        rowspan,
                    });
                }
            }
        }

        if !cells.is_empty() {
            output.push(LayoutElement::TableRow {
                cells,
                col_width,
                margin_top: if is_first { style.margin.top } else { 0.0 },
                margin_bottom: 0.0,
            });
            is_first = false;
        }
    }

    // Add bottom margin after the last row
    if let Some(LayoutElement::TableRow { margin_bottom, .. }) = output.last_mut() {
        *margin_bottom = style.margin.bottom;
    }
}

fn collect_text_runs(
    nodes: &[DomNode],
    parent_style: &ComputedStyle,
    runs: &mut Vec<TextRun>,
    link_url: Option<&str>,
) {
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
                        line_through: parent_style.text_decoration_line_through,
                        color: parent_style.color.to_f32_rgb(),
                        link_url: link_url.map(String::from),
                        font_family: parent_style.font_family,
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
                            line_through: false,
                            color: (0.0, 0.0, 0.0),
                            link_url: None,
                            font_family: parent_style.font_family,
                        });
                    } else {
                        let style = compute_style(el.tag, el.style_attr(), parent_style);
                        let url = if el.tag == HtmlTag::A {
                            el.attributes.get("href").map(|s| s.as_str()).or(link_url)
                        } else {
                            link_url
                        };
                        collect_text_runs(&el.children, &style, runs, url);
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
            LayoutElement::TableRow {
                cells,
                margin_top,
                margin_bottom,
                ..
            } => {
                let row_height = cells
                    .iter()
                    .map(|cell| {
                        let text_h: f32 = cell.lines.iter().map(|l| l.height).sum();
                        cell.padding_top + text_h + cell.padding_bottom
                    })
                    .fold(0.0f32, f32::max);
                (margin_top + row_height + margin_bottom, *margin_top)
            }
            LayoutElement::TextBlock {
                lines,
                margin_top,
                margin_bottom,
                padding_top,
                padding_bottom,
                border_width,
                ..
            } => {
                let text_height: f32 = lines.iter().map(|l| l.height).sum();
                let border_extra = border_width * 2.0;
                let total = margin_top
                    + padding_top
                    + text_height
                    + padding_bottom
                    + margin_bottom
                    + border_extra;
                (total, *margin_top)
            }
            LayoutElement::Image {
                height,
                margin_top,
                margin_bottom,
                ..
            } => (*margin_top + *height + *margin_bottom, *margin_top),
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

/// Decode a base64 string into bytes. Ignores whitespace. Returns None on invalid input.
fn base64_decode(input: &str) -> Option<Vec<u8>> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut buf = Vec::new();
    let mut accum: u32 = 0;
    let mut bits: u32 = 0;
    for &b in input.as_bytes() {
        if b == b'=' || b.is_ascii_whitespace() {
            continue;
        }
        let val = TABLE.iter().position(|&c| c == b)? as u32;
        accum = (accum << 6) | val;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            buf.push((accum >> bits) as u8);
            accum &= (1 << bits) - 1;
        }
    }
    Some(buf)
}

/// Load image data from an `<img>` element's `src` attribute.
/// Supports `data:image/jpeg;base64,...` URIs and local file paths.
fn load_image_data(el: &ElementNode) -> Option<Vec<u8>> {
    let src = el.attributes.get("src")?;
    if src.starts_with("data:") {
        // data URI: data:image/jpeg;base64,<data>
        let comma_pos = src.find(',')?;
        let encoded = &src[comma_pos + 1..];
        base64_decode(encoded)
    } else {
        // Treat as local file path
        std::fs::read(src).ok()
    }
}

/// Parse width/height attributes from an `<img>` element.
/// Converts px values to pt (*0.75). Defaults to 200x150 pt.
fn parse_image_dimensions(el: &ElementNode) -> (f32, f32) {
    let default_w = 200.0_f32;
    let default_h = 150.0_f32;

    let w = el
        .attributes
        .get("width")
        .and_then(|v| v.trim_end_matches("px").parse::<f32>().ok())
        .map(|px| px * 0.75)
        .unwrap_or(default_w);
    let h = el
        .attributes
        .get("height")
        .and_then(|v| v.trim_end_matches("px").parse::<f32>().ok())
        .map(|px| px * 0.75)
        .unwrap_or(default_h);
    (w, h)
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

    #[test]
    fn table_layout_basic() {
        // Exercises flatten_table and table row layout (lines 232, 248, 344, 354)
        let html = r#"
            <table>
                <tr><th>Header 1</th><th>Header 2</th></tr>
                <tr><td>Cell A</td><td>Cell B</td></tr>
            </table>
        "#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        // Should have TableRow elements
        let table_rows: Vec<_> = pages[0]
            .elements
            .iter()
            .filter(|(_, el)| matches!(el, LayoutElement::TableRow { .. }))
            .collect();
        assert_eq!(table_rows.len(), 2);
    }

    #[test]
    fn table_with_thead_tbody_tfoot() {
        // Exercises lines 345-353: collecting rows from thead/tbody/tfoot
        let html = r#"
            <table>
                <thead><tr><th>H</th></tr></thead>
                <tbody><tr><td>B</td></tr></tbody>
                <tfoot><tr><td>F</td></tr></tfoot>
            </table>
        "#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        let table_rows: Vec<_> = pages[0]
            .elements
            .iter()
            .filter(|(_, el)| matches!(el, LayoutElement::TableRow { .. }))
            .collect();
        assert_eq!(table_rows.len(), 3);
    }

    #[test]
    fn table_empty_rows_ignored() {
        // Line 360: empty table returns early
        let html = "<table></table>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        // Should have no table rows
        let table_rows: Vec<_> = pages[0]
            .elements
            .iter()
            .filter(|(_, el)| matches!(el, LayoutElement::TableRow { .. }))
            .collect();
        assert_eq!(table_rows.len(), 0);
    }

    #[test]
    fn ordered_list_layout() {
        // Exercises lines 219-232, 248: ordered list context and numbering
        let html = "<ol><li>First</li><li>Second</li><li>Third</li></ol>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        // Should have items with numbered markers
        let blocks: Vec<_> = pages[0]
            .elements
            .iter()
            .filter(|(_, el)| matches!(el, LayoutElement::TextBlock { .. }))
            .collect();
        assert!(blocks.len() >= 3);
    }

    #[test]
    fn unordered_list_layout() {
        // Exercises lines 217-236: unordered list layout
        let html = "<ul><li>A</li><li>B</li></ul>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        assert!(!pages[0].elements.is_empty());
    }

    #[test]
    fn list_with_non_li_child() {
        // Line 232: non-li child inside ul
        let html = "<ul><li>Item</li><p>Not a list item</p></ul>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
    }

    #[test]
    fn li_with_block_child() {
        // Lines 279-280: block child inside li
        let html = "<ul><li><p>Paragraph inside li</p></li></ul>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        assert!(!pages[0].elements.is_empty());
    }

    #[test]
    fn table_row_pagination() {
        // Exercises TableRow height calculation in paginate (lines 559-572)
        let mut rows = String::new();
        for i in 0..100 {
            rows.push_str(&format!(
                "<tr><td>Row {i} with some text</td><td>More text</td></tr>"
            ));
        }
        let html = format!("<table>{rows}</table>");
        let nodes = parse_html(&html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert!(pages.len() >= 2, "Large table should span multiple pages");
    }

    #[test]
    fn table_with_non_cell_children_in_row() {
        // Line 354: non-td/th child in tr is ignored
        let html = r#"<table><tr><td>Cell</td><span>Ignored</span></tr></table>"#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        let table_rows: Vec<_> = pages[0]
            .elements
            .iter()
            .filter(|(_, el)| matches!(el, LayoutElement::TableRow { .. }))
            .collect();
        assert_eq!(table_rows.len(), 1);
    }

    #[test]
    fn del_element_sets_line_through() {
        let html = "<p><del>Deleted text</del></p>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        if let (_, LayoutElement::TextBlock { lines, .. }) = &pages[0].elements[0] {
            assert!(!lines.is_empty());
            let run = &lines[0].runs[0];
            assert!(run.line_through, "del element should set line_through");
            assert!(!run.underline);
        } else {
            panic!("Expected TextBlock");
        }
    }

    #[test]
    fn s_element_sets_line_through() {
        let html = "<p><s>Struck text</s></p>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        if let (_, LayoutElement::TextBlock { lines, .. }) = &pages[0].elements[0] {
            assert!(!lines.is_empty());
            let run = &lines[0].runs[0];
            assert!(run.line_through, "s element should set line_through");
        } else {
            panic!("Expected TextBlock");
        }
    }

    #[test]
    fn nested_unordered_list() {
        let html = "<ul><li>Parent<ul><li>Child</li></ul></li></ul>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        // Should have at least 2 TextBlock elements: parent item and nested child item
        let blocks: Vec<_> = pages[0]
            .elements
            .iter()
            .filter_map(|(_, el)| match el {
                LayoutElement::TextBlock {
                    lines,
                    padding_left,
                    ..
                } => Some((lines.clone(), *padding_left)),
                _ => None,
            })
            .collect();
        assert!(
            blocks.len() >= 2,
            "Expected at least 2 text blocks for nested list, got {}",
            blocks.len()
        );
        // The nested item should have greater indentation than the parent
        let parent_indent = blocks[0].1;
        let child_indent = blocks[1].1;
        assert!(
            child_indent > parent_indent,
            "Nested list item should be more indented: parent={parent_indent}, child={child_indent}"
        );
    }

    #[test]
    fn nested_ordered_list() {
        let html = "<ol><li>First<ol><li>Nested first</li><li>Nested second</li></ol></li><li>Second</li></ol>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        let blocks: Vec<_> = pages[0]
            .elements
            .iter()
            .filter_map(|(_, el)| match el {
                LayoutElement::TextBlock {
                    lines,
                    padding_left,
                    ..
                } => Some((lines.clone(), *padding_left)),
                _ => None,
            })
            .collect();
        // Should have: "1. First", "1. Nested first", "2. Nested second", "2. Second"
        assert!(
            blocks.len() >= 3,
            "Expected at least 3 text blocks for nested ordered list, got {}",
            blocks.len()
        );
        // Nested items should have greater indentation
        let parent_indent = blocks[0].1;
        let nested_indent = blocks[1].1;
        assert!(
            nested_indent > parent_indent,
            "Nested ordered list should be more indented: parent={parent_indent}, nested={nested_indent}"
        );
    }

    #[test]
    fn mixed_nested_list() {
        let html = "<ul><li>Bullet<ol><li>Numbered</li></ol></li></ul>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        let blocks: Vec<_> = pages[0]
            .elements
            .iter()
            .filter_map(|(_, el)| match el {
                LayoutElement::TextBlock {
                    lines,
                    padding_left,
                    ..
                } => Some((lines.clone(), *padding_left)),
                _ => None,
            })
            .collect();
        assert!(
            blocks.len() >= 2,
            "Expected at least 2 text blocks for mixed nested list, got {}",
            blocks.len()
        );
        // Nested ordered list inside unordered should be more indented
        let parent_indent = blocks[0].1;
        let nested_indent = blocks[1].1;
        assert!(
            nested_indent > parent_indent,
            "Nested ol inside ul should be more indented: parent={parent_indent}, nested={nested_indent}"
        );
        // Check that the nested item has a numbered marker
        let nested_text: String = blocks[1].0[0].runs.iter().map(|r| r.text.clone()).collect();
        assert!(
            nested_text.contains("1."),
            "Nested item should have ordered marker, got: {nested_text}"
        );
    }

    #[test]
    fn base64_decode_basic() {
        // "Hello" in base64 is "SGVsbG8="
        let decoded = super::base64_decode("SGVsbG8=").unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn base64_decode_with_whitespace() {
        let decoded = super::base64_decode("SGVs\nbG8=").unwrap();
        assert_eq!(decoded, b"Hello");
    }

    #[test]
    fn img_data_uri_creates_image_element() {
        // Minimal valid JPEG: SOI (FF D8) + APP0 marker + EOI (FF D9)
        // We encode a tiny JPEG-like byte sequence in base64
        // FF D8 FF E0 00 02 FF D9 => /9j/4AAC/9k=
        let html = r#"<img src="data:image/jpeg;base64,/9j/4AAC/9k=" width="100" height="80">"#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        let has_image = pages[0]
            .elements
            .iter()
            .any(|(_, el)| matches!(el, LayoutElement::Image { .. }));
        assert!(has_image, "Should contain an Image layout element");
    }

    #[test]
    fn img_dimensions_converted_to_pt() {
        let html = r#"<img src="data:image/jpeg;base64,/9j/4AAC/9k=" width="400" height="300">"#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        if let (_, LayoutElement::Image { width, height, .. }) = &pages[0].elements[0] {
            // 400px * 0.75 = 300pt, 300px * 0.75 = 225pt
            assert!((*width - 300.0).abs() < 0.01);
            assert!((*height - 225.0).abs() < 0.01);
        } else {
            panic!("Expected Image element");
        }
    }

    #[test]
    fn img_scales_to_fit_available_width() {
        // Very wide image: 2000px = 1500pt, which exceeds A4 content width (~451pt)
        let html = r#"<img src="data:image/jpeg;base64,/9j/4AAC/9k=" width="2000" height="1000">"#;
        let nodes = parse_html(html).unwrap();
        let page_size = PageSize::A4;
        let margin_val = Margin::default();
        let available_width = page_size.width - margin_val.left - margin_val.right;
        let pages = layout(&nodes, page_size, margin_val);
        if let (_, LayoutElement::Image { width, .. }) = &pages[0].elements[0] {
            assert!(
                *width <= available_width + 0.01,
                "Image width {width} should fit within available width {available_width}"
            );
        } else {
            panic!("Expected Image element");
        }
    }

    #[test]
    fn img_without_src_ignored() {
        let html = r#"<img width="100" height="80">"#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        let has_image = pages[0]
            .elements
            .iter()
            .any(|(_, el)| matches!(el, LayoutElement::Image { .. }));
        assert!(
            !has_image,
            "img without src should not produce Image element"
        );
    }

    #[test]
    fn three_levels_deep_nested_list() {
        let html = "<ul><li>Level 1<ul><li>Level 2<ul><li>Level 3</li></ul></li></ul></li></ul>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        assert_eq!(pages.len(), 1);
        let blocks: Vec<_> = pages[0]
            .elements
            .iter()
            .filter_map(|(_, el)| match el {
                LayoutElement::TextBlock {
                    lines,
                    padding_left,
                    ..
                } => Some((lines.clone(), *padding_left)),
                _ => None,
            })
            .collect();
        assert!(
            blocks.len() >= 3,
            "Expected at least 3 text blocks for 3-level list, got {}",
            blocks.len()
        );
        let indent_1 = blocks[0].1;
        let indent_2 = blocks[1].1;
        let indent_3 = blocks[2].1;
        assert!(
            indent_2 > indent_1,
            "Level 2 should be more indented than level 1: l1={indent_1}, l2={indent_2}"
        );
        assert!(
            indent_3 > indent_2,
            "Level 3 should be more indented than level 2: l2={indent_2}, l3={indent_3}"
        );
    }

    #[test]
    fn table_colspan_default_is_one() {
        let html = "<table><tr><td>A</td><td>B</td></tr></table>";
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        for (_, el) in &pages[0].elements {
            if let LayoutElement::TableRow { cells, .. } = el {
                for cell in cells {
                    assert_eq!(cell.colspan, 1, "Default colspan should be 1");
                    assert_eq!(cell.rowspan, 1, "Default rowspan should be 1");
                }
            }
        }
    }

    #[test]
    fn table_colspan_header_spans_two() {
        let html =
            r#"<table><tr><th colspan="2">Header</th></tr><tr><td>A</td><td>B</td></tr></table>"#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        let table_rows: Vec<_> = pages[0]
            .elements
            .iter()
            .filter_map(|(_, el)| {
                if let LayoutElement::TableRow { cells, .. } = el {
                    Some(cells)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(table_rows.len(), 2);
        assert_eq!(table_rows[0].len(), 1);
        assert_eq!(table_rows[0][0].colspan, 2);
        assert_eq!(table_rows[1].len(), 2);
        assert_eq!(table_rows[1][0].colspan, 1);
        assert_eq!(table_rows[1][1].colspan, 1);
    }

    #[test]
    fn table_colspan_makes_cells_wider() {
        let html = r#"<table><tr><td colspan="2">Wide</td><td>N</td></tr><tr><td>A</td><td>B</td><td>C</td></tr></table>"#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        let table_rows: Vec<_> = pages[0]
            .elements
            .iter()
            .filter_map(|(_, el)| {
                if let LayoutElement::TableRow {
                    cells, col_width, ..
                } = el
                {
                    Some((cells, *col_width))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(table_rows.len(), 2);
        let (cells, col_width) = &table_rows[0];
        assert_eq!(cells[0].colspan, 2);
        let available = PageSize::A4.width - Margin::default().left - Margin::default().right;
        let expected_col_w = available / 3.0;
        assert!(
            (col_width - expected_col_w).abs() < 0.1,
            "col_width should reflect 3 columns: got {col_width}, expected {expected_col_w}"
        );
    }

    #[test]
    fn table_mixed_colspan_values() {
        let html = r#"<table><tr><td colspan="3">Full</td></tr><tr><td>A</td><td colspan="2">BC</td></tr><tr><td>X</td><td>Y</td><td>Z</td></tr></table>"#;
        let nodes = parse_html(html).unwrap();
        let pages = layout(&nodes, PageSize::A4, Margin::default());
        let table_rows: Vec<_> = pages[0]
            .elements
            .iter()
            .filter_map(|(_, el)| {
                if let LayoutElement::TableRow { cells, .. } = el {
                    Some(cells)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(table_rows.len(), 3);
        assert_eq!(table_rows[0].len(), 1);
        assert_eq!(table_rows[0][0].colspan, 3);
        assert_eq!(table_rows[1].len(), 2);
        assert_eq!(table_rows[1][0].colspan, 1);
        assert_eq!(table_rows[1][1].colspan, 2);
        assert_eq!(table_rows[2].len(), 3);
        for cell in table_rows[2] {
            assert_eq!(cell.colspan, 1);
        }
    }
}
