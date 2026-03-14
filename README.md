# ironpress

[![Crates.io](https://img.shields.io/crates/v/ironpress.svg)](https://crates.io/crates/ironpress)
[![docs.rs](https://docs.rs/ironpress/badge.svg)](https://docs.rs/ironpress)
[![CI](https://github.com/gastongouron/ironpress/actions/workflows/ci.yml/badge.svg)](https://github.com/gastongouron/ironpress/actions)
[![codecov](https://codecov.io/gh/gastongouron/ironpress/branch/main/graph/badge.svg)](https://codecov.io/gh/gastongouron/ironpress)

Pure Rust HTML/CSS-to-PDF converter — no browser, no external dependencies.

Every existing Rust crate that converts HTML to PDF shells out to headless Chrome or wkhtmltopdf. **ironpress** does it natively with a built-in layout engine, producing valid PDFs from HTML with inline CSS.

## Quick Start

```rust
use ironpress::html_to_pdf;

let pdf_bytes = html_to_pdf("<h1>Hello</h1><p>World</p>").unwrap();
std::fs::write("output.pdf", pdf_bytes).unwrap();
```

## With Options

```rust
use ironpress::{HtmlConverter, PageSize, Margin};

let pdf = HtmlConverter::new()
    .page_size(PageSize::LETTER)
    .margin(Margin::uniform(54.0))
    .convert("<h1>Custom page</h1>")
    .unwrap();
```

## File Conversion

```rust
ironpress::convert_file("input.html", "output.pdf").unwrap();
```

## Supported HTML Elements

| Element | Rendering |
|---------|-----------|
| `<h1>` - `<h6>` | Headings with default sizes and bold |
| `<p>`, `<div>` | Block containers |
| `<strong>`, `<b>` | Bold text |
| `<em>`, `<i>` | Italic text |
| `<u>` | Underlined text |
| `<a>` | Colored underlined text |
| `<br>` | Line break |
| `<hr>` | Horizontal rule |
| `<ul>`, `<ol>`, `<li>` | Lists |
| `<table>`, `<tr>`, `<td>`, `<th>` | Tables |
| `<span>` | Inline container |

## Supported CSS Properties (inline `style="..."`)

`font-size`, `font-weight`, `font-style`, `color`, `background-color`, `margin`, `padding`, `text-align`, `text-decoration`, `line-height`, `page-break-before`, `page-break-after`

Colors: named colors, `#hex`, `rgb()`.

## Security

HTML is sanitized by default before conversion:

- `<script>`, `<iframe>`, `<object>`, `<embed>`, `<form>` tags are stripped
- Event handlers (`onclick`, `onload`, etc.) are removed
- `javascript:` URLs are neutralized
- Input size and nesting depth are limited

Sanitization can be disabled with `.sanitize(false)` if you trust the input.

## How It Works

```
HTML string → Parse (html5ever) → Style resolution → Layout engine → PDF generation
```

1. **Parse** HTML into a DOM tree using html5ever
2. **Resolve styles** by merging default tag styles with inline CSS
3. **Layout** elements with text wrapping, page breaks, and box model
4. **Render** to PDF using built-in Helvetica fonts (no font embedding needed)

## License

MIT
