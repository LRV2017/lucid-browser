//! Lucid Engine: an experimental HTML/CSS parsing and layout prototype.
//!
//! This crate intentionally does not embed Gecko, Blink, WebKit, CEF, WebView2,
//! or another browser engine. The parser, simple cascade, block layout and paint
//! command generation in this crate are Lucid-owned prototype implementations.
//! It is not HTML5/CSS conformance complete and does not execute JavaScript.

use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Document {
    pub children: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Element(Element),
    Text(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Element {
    pub tag: String,
    pub attributes: BTreeMap<String, String>,
    pub children: Vec<Node>,
}

impl Element {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attributes.get(&name.to_ascii_lowercase()).map(String::as_str)
    }
}

fn is_void(tag: &str) -> bool {
    matches!(tag, "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta" | "param" | "source" | "track" | "wbr")
}

fn append_node(stack: &mut [Element], roots: &mut Vec<Node>, node: Node) {
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}

/// Parse a useful HTML subset with a small, Lucid-owned tokenizer/tree builder.
/// Error recovery is deliberately conservative; full WHATWG tree construction
/// (foster parenting, adoption agency algorithm, implied end tags, etc.) is future work.
pub fn parse_html(source: &str) -> Document {
    let chars: Vec<char> = source.chars().collect();
    let mut cursor = 0usize;
    let mut stack: Vec<Element> = Vec::new();
    let mut roots = Vec::new();

    while cursor < chars.len() {
        if chars[cursor] != '<' {
            let start = cursor;
            while cursor < chars.len() && chars[cursor] != '<' {
                cursor += 1;
            }
            let text: String = chars[start..cursor].iter().collect();
            if !text.is_empty() {
                append_node(&mut stack, &mut roots, Node::Text(decode_entities(&text)));
            }
            continue;
        }

        if starts_with(&chars, cursor, "<!--") {
            cursor = find_sequence(&chars, cursor + 4, "-->").unwrap_or(chars.len());
            cursor = (cursor + 3).min(chars.len());
            continue;
        }

        if starts_with(&chars, cursor, "<!") || starts_with(&chars, cursor, "<?") {
            cursor = find_char(&chars, cursor, '>').map_or(chars.len(), |i| i + 1);
            continue;
        }

        if starts_with(&chars, cursor, "</") {
            cursor += 2;
            skip_space(&chars, &mut cursor);
            let tag = read_name(&chars, &mut cursor).to_ascii_lowercase();
            cursor = find_char(&chars, cursor, '>').map_or(chars.len(), |i| i + 1);
            if !tag.is_empty() {
                while let Some(element) = stack.pop() {
                    let matched = element.tag == tag;
                    append_node(&mut stack, &mut roots, Node::Element(element));
                    if matched {
                        break;
                    }
                }
            }
            continue;
        }

        // A '<' that is not followed by a tag name is treated as text.
        let tag_start = cursor + 1;
        if tag_start >= chars.len() || !is_name_char(chars[tag_start]) {
            append_node(&mut stack, &mut roots, Node::Text("<".to_owned()));
            cursor += 1;
            continue;
        }

        cursor += 1;
        let tag = read_name(&chars, &mut cursor).to_ascii_lowercase();
        let mut attributes = BTreeMap::new();
        let mut self_closing = false;

        loop {
            skip_space(&chars, &mut cursor);
            if cursor >= chars.len() {
                break;
            }
            match chars[cursor] {
                '>' => {
                    cursor += 1;
                    break;
                }
                '/' if chars.get(cursor + 1) == Some(&'>') => {
                    self_closing = true;
                    cursor += 2;
                    break;
                }
                _ => {
                    let name = read_attribute_name(&chars, &mut cursor).to_ascii_lowercase();
                    if name.is_empty() {
                        cursor += 1;
                        continue;
                    }
                    skip_space(&chars, &mut cursor);
                    let value = if chars.get(cursor) == Some(&'=') {
                        cursor += 1;
                        skip_space(&chars, &mut cursor);
                        read_attribute_value(&chars, &mut cursor)
                    } else {
                        String::new()
                    };
                    attributes.insert(name, decode_entities(&value));
                }
            }
        }

        // Handle common omitted paragraph/list-item endings without attempting
        // the complete HTML5 parsing algorithm.
        if matches!(tag.as_str(), "p" | "li" | "dt" | "dd") {
            if stack.last().is_some_and(|parent| parent.tag == tag) {
                if let Some(element) = stack.pop() {
                    append_node(&mut stack, &mut roots, Node::Element(element));
                }
            }
        }

        let element = Element { tag: tag.clone(), attributes, children: Vec::new() };
        if self_closing || is_void(&tag) {
            append_node(&mut stack, &mut roots, Node::Element(element));
        } else {
            stack.push(element);
        }
    }

    while let Some(element) = stack.pop() {
        append_node(&mut stack, &mut roots, Node::Element(element));
    }

    Document { children: roots }
}

fn starts_with(chars: &[char], at: usize, needle: &str) -> bool {
    let needle: Vec<char> = needle.chars().collect();
    chars.get(at..at.saturating_add(needle.len())) == Some(needle.as_slice())
}

fn find_sequence(chars: &[char], from: usize, needle: &str) -> Option<usize> {
    (from..chars.len()).find(|&i| starts_with(chars, i, needle))
}

fn find_char(chars: &[char], from: usize, target: char) -> Option<usize> {
    (from..chars.len()).find(|&i| chars[i] == target)
}

fn skip_space(chars: &[char], cursor: &mut usize) {
    while *cursor < chars.len() && chars[*cursor].is_whitespace() {
        *cursor += 1;
    }
}

fn is_name_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | ':' | '.')
}

fn read_name(chars: &[char], cursor: &mut usize) -> String {
    let start = *cursor;
    while *cursor < chars.len() && is_name_char(chars[*cursor]) {
        *cursor += 1;
    }
    chars[start..*cursor].iter().collect()
}

fn read_attribute_name(chars: &[char], cursor: &mut usize) -> String {
    let start = *cursor;
    while *cursor < chars.len() && !chars[*cursor].is_whitespace() && !matches!(chars[*cursor], '=' | '>' | '/') {
        *cursor += 1;
    }
    chars[start..*cursor].iter().collect()
}

fn read_attribute_value(chars: &[char], cursor: &mut usize) -> String {
    if *cursor >= chars.len() {
        return String::new();
    }
    let quote = chars[*cursor];
    if quote == '\'' || quote == '"' {
        *cursor += 1;
        let start = *cursor;
        while *cursor < chars.len() && chars[*cursor] != quote {
            *cursor += 1;
        }
        let value: String = chars[start..*cursor].iter().collect();
        if *cursor < chars.len() { *cursor += 1; }
        value
    } else {
        let start = *cursor;
        while *cursor < chars.len() && !chars[*cursor].is_whitespace() && chars[*cursor] != '>' {
            *cursor += 1;
        }
        chars[start..*cursor].iter().collect()
    }
}

fn decode_entities(input: &str) -> String {
    input
        .replace("&nbsp;", "\u{00a0}")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rule {
    pub selector: String,
    pub declarations: Style,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub display: Display,
    pub color: Option<String>,
    pub background: Option<String>,
    pub font_size: f32,
    pub bold: bool,
    pub italic: bool,
    pub margin: f32,
    pub padding: f32,
    pub radius: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self { display: Display::Block, color: None, background: None, font_size: 16.0, bold: false, italic: false, margin: 4.0, padding: 0.0, radius: 0.0 }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Display {
    #[default]
    Block,
    Inline,
    None,
}

pub fn parse_stylesheet(source: &str) -> Stylesheet {
    let mut sheet = Stylesheet::default();
    for rule_text in source.split('}') {
        let Some((selector_text, declarations_text)) = rule_text.split_once('{') else { continue };
        let style = parse_declarations(declarations_text);
        for selector in selector_text.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            sheet.rules.push(Rule { selector: selector.to_owned(), declarations: style.clone() });
        }
    }
    sheet
}

fn parse_declarations(source: &str) -> Style {
    let mut style = Style::default();
    for declaration in source.split(';') {
        let Some((property, value)) = declaration.split_once(':') else { continue };
        let property = property.trim().to_ascii_lowercase();
        let value = value.trim();
        match property.as_str() {
            "display" => style.display = match value { "none" => Display::None, "inline" | "inline-block" => Display::Inline, _ => Display::Block },
            "color" => style.color = Some(value.to_owned()),
            "background" | "background-color" => style.background = Some(value.to_owned()),
            "font-size" => if let Some(n) = parse_px(value) { style.font_size = n.clamp(7.0, 96.0); },
            "font-weight" => style.bold = value == "bold" || value.parse::<u16>().is_ok_and(|n| n >= 600),
            "font-style" => style.italic = value == "italic",
            "margin" | "padding" => if let Some(n) = parse_px(value) { if property == "margin" { style.margin = n.clamp(0.0, 120.0); } else { style.padding = n.clamp(0.0, 120.0); } },
            "border-radius" => if let Some(n) = parse_px(value) { style.radius = n.clamp(0.0, 48.0); },
            _ => {}
        }
    }
    style
}

fn parse_px(value: &str) -> Option<f32> {
    value.trim_end_matches("px").trim().parse::<f32>().ok()
}

fn selector_specificity(selector: &str) -> u8 {
    if selector.starts_with('#') { 3 } else if selector.starts_with('.') { 2 } else { 1 }
}

fn selector_matches(selector: &str, element: &Element) -> bool {
    let selector = selector.trim();
    if let Some(id) = selector.strip_prefix('#') {
        element.attr("id") == Some(id)
    } else if let Some(class) = selector.strip_prefix('.') {
        element.attr("class").is_some_and(|classes| classes.split_whitespace().any(|c| c == class))
    } else {
        selector.eq_ignore_ascii_case(&element.tag)
    }
}

fn merge_style(base: &mut Style, add: &Style) {
    if add.display != Display::Block || base.display == Display::Block { base.display = add.display; }
    if add.color.is_some() { base.color = add.color.clone(); }
    if add.background.is_some() { base.background = add.background.clone(); }
    if (add.font_size - 16.0).abs() > f32::EPSILON { base.font_size = add.font_size; }
    base.bold |= add.bold;
    base.italic |= add.italic;
    if add.margin != 4.0 { base.margin = add.margin; }
    if add.padding != 0.0 { base.padding = add.padding; }
    if add.radius != 0.0 { base.radius = add.radius; }
}

fn default_style(tag: &str) -> Style {
    let mut style = Style::default();
    style.margin = if matches!(tag, "p" | "h1" | "h2" | "h3" | "ul" | "ol" | "pre" | "blockquote") { 12.0 } else { 3.0 };
    match tag {
        "h1" => { style.font_size = 32.0; style.bold = true; }
        "h2" => { style.font_size = 26.0; style.bold = true; }
        "h3" => { style.font_size = 21.0; style.bold = true; }
        "h4" | "strong" | "b" => style.bold = true,
        "em" | "i" => style.italic = true,
        "a" => style.color = Some("#A98BEA".to_owned()),
        "small" => style.font_size = 13.0,
        "code" | "pre" => style.background = Some("#24202d".to_owned()),
        "br" => style.margin = 0.0,
        _ => {}
    }
    if matches!(tag, "span" | "a" | "strong" | "b" | "em" | "i" | "code" | "small") { style.display = Display::Inline; }
    style
}

fn style_for(element: &Element, sheet: &Stylesheet) -> Style {
    let mut style = default_style(&element.tag);
    let mut matches: Vec<&Rule> = sheet.rules.iter().filter(|r| selector_matches(&r.selector, element)).collect();
    matches.sort_by_key(|r| selector_specificity(&r.selector));
    for rule in matches { merge_style(&mut style, &rule.declarations); }
    if let Some(inline) = element.attr("style") { merge_style(&mut style, &parse_declarations(inline)); }
    style
}

pub fn collect_styles(document: &Document) -> Stylesheet {
    fn walk(nodes: &[Node], output: &mut String) {
        for node in nodes {
            if let Node::Element(el) = node {
                if el.tag == "style" {
                    for child in &el.children { if let Node::Text(text) = child { output.push_str(text); output.push('\n'); } }
                } else {
                    walk(&el.children, output);
                }
            }
        }
    }
    let mut source = String::new();
    walk(&document.children, &mut source);
    parse_stylesheet(&source)
}

pub fn document_title(document: &Document) -> Option<String> {
    fn walk(nodes: &[Node]) -> Option<String> {
        for node in nodes {
            if let Node::Element(el) = node {
                if el.tag == "title" {
                    let mut text = String::new();
                    collect_text(&el.children, &mut text);
                    let title = text.split_whitespace().collect::<Vec<_>>().join(" ");
                    if !title.is_empty() { return Some(title); }
                }
                if let Some(found) = walk(&el.children) { return Some(found); }
            }
        }
        None
    }
    walk(&document.children)
}

fn collect_text(nodes: &[Node], output: &mut String) {
    for node in nodes {
        match node { Node::Text(text) => output.push_str(text), Node::Element(el) => collect_text(&el.children, output) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaintOp {
    Rect { x: f32, y: f32, width: f32, height: f32, color: Color, radius: f32 },
    Text { x: f32, y: f32, text: String, size: f32, color: Color, bold: bool, italic: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub const TEXT: Self = Self(38, 35, 44);
    pub const MUTED: Self = Self(113, 107, 122);
    pub const LINK: Self = Self(117, 78, 190);
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutOutput {
    pub operations: Vec<PaintOp>,
    pub height: f32,
}

/// Produce a simple top-to-bottom block layout and paint list.
/// This is a deliberate first milestone, not a CSS formatting-context implementation.
pub fn layout_document(document: &Document, sheet: &Stylesheet, viewport_width: f32) -> LayoutOutput {
    let mut output = LayoutOutput::default();
    let width = viewport_width.max(240.0);
    let mut y = 24.0;
    layout_nodes(&document.children, sheet, 24.0, &mut y, width - 48.0, &mut output.operations, Style::default());
    output.height = (y + 40.0).max(240.0);
    output
}

fn layout_nodes(nodes: &[Node], sheet: &Stylesheet, x: f32, y: &mut f32, width: f32, ops: &mut Vec<PaintOp>, inherited: Style) {
    for node in nodes {
        match node {
            Node::Text(text) => layout_text(text, x, y, width, &inherited, ops),
            Node::Element(element) => {
                if matches!(element.tag.as_str(), "head" | "script" | "style" | "meta" | "link" | "title") { continue; }
                let style = style_for(element, sheet);
                if style.display == Display::None { continue; }
                if element.tag == "br" { *y += style.font_size * 1.35; continue; }
                if element.tag == "hr" {
                    ops.push(PaintOp::Rect { x, y: *y + 5.0, width, height: 1.0, color: Color(218, 214, 225), radius: 0.0 });
                    *y += 16.0;
                    continue;
                }
                let pad = style.padding;
                *y += style.margin;
                if let Some(background) = style.background.as_deref().and_then(parse_color) {
                    let estimated_height = estimate_text_height(&element.children, width - pad * 2.0, style.font_size) + pad * 2.0;
                    ops.push(PaintOp::Rect { x, y: *y, width, height: estimated_height.max(24.0), color: background, radius: style.radius });
                }
                let before = *y;
                *y += pad;
                let child_x = x + pad;
                let child_width = (width - pad * 2.0).max(40.0);
                let mut child_style = style.clone();
                if element.tag == "a" { child_style.color = Some("#7650BD".to_owned()); }
                layout_nodes(&element.children, sheet, child_x, y, child_width, ops, child_style);
                if element.tag == "img" {
                    let alt = element.attr("alt").unwrap_or("Image");
                    ops.push(PaintOp::Text { x: child_x, y: *y, text: format!("[ {alt} ]"), size: 13.0, color: Color::MUTED, bold: false, italic: true });
                    *y += 24.0;
                }
                *y += pad;
                if *y == before + pad { *y += 20.0; }
                *y += style.margin;
            }
        }
    }
}

fn estimate_text_height(nodes: &[Node], width: f32, font_size: f32) -> f32 {
    let mut text = String::new();
    collect_text(nodes, &mut text);
    let chars_per_line = (width / (font_size * 0.55)).max(8.0) as usize;
    let lines = text.split_whitespace().count().max(1).div_ceil(chars_per_line.max(1));
    lines as f32 * font_size * 1.5
}

fn layout_text(text: &str, x: f32, y: &mut f32, width: f32, style: &Style, ops: &mut Vec<PaintOp>) {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() { return; }
    let chars_per_line = (width / (style.font_size * 0.55)).max(8.0) as usize;
    let mut line = String::new();
    for word in normalized.split(' ') {
        let candidate = if line.is_empty() { word.to_owned() } else { format!("{line} {word}") };
        if candidate.chars().count() > chars_per_line && !line.is_empty() {
            push_text_line(&line, x, y, style, ops);
            line = word.to_owned();
        } else { line = candidate; }
    }
    if !line.is_empty() { push_text_line(&line, x, y, style, ops); }
}

fn push_text_line(line: &str, x: f32, y: &mut f32, style: &Style, ops: &mut Vec<PaintOp>) {
    let color = style.color.as_deref().and_then(parse_color).unwrap_or(Color::TEXT);
    ops.push(PaintOp::Text { x, y: *y, text: line.to_owned(), size: style.font_size, color, bold: style.bold, italic: style.italic });
    *y += style.font_size * 1.45;
}

pub fn parse_color(input: &str) -> Option<Color> {
    let value = input.trim().to_ascii_lowercase();
    if let Some(hex) = value.strip_prefix('#') {
        let full = if hex.len() == 3 { hex.chars().flat_map(|c| [c, c]).collect::<String>() } else { hex.to_owned() };
        if full.len() == 6 {
            let r = u8::from_str_radix(&full[0..2], 16).ok()?;
            let g = u8::from_str_radix(&full[2..4], 16).ok()?;
            let b = u8::from_str_radix(&full[4..6], 16).ok()?;
            return Some(Color(r, g, b));
        }
    }
    match value.as_str() {
        "black" => Some(Color(0, 0, 0)), "white" => Some(Color(255, 255, 255)),
        "red" => Some(Color(220, 35, 50)), "green" => Some(Color(30, 145, 75)),
        "blue" => Some(Color(45, 100, 210)), "gray" | "grey" => Some(Color(128, 128, 128)),
        "purple" => Some(Color(128, 60, 180)), "orange" => Some(Color(225, 125, 35)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_elements_and_attributes() {
        let doc = parse_html(r#"<main id='root'><h1 class="hero">Hello</h1><p>world &amp; friends</p></main>"#);
        assert_eq!(doc.children.len(), 1);
        let Node::Element(main) = &doc.children[0] else { panic!("expected main") };
        assert_eq!(main.tag, "main");
        assert_eq!(main.attr("id"), Some("root"));
        assert_eq!(main.children.len(), 2);
    }

    #[test]
    fn reads_title_and_styles() {
        let doc = parse_html("<html><head><title>Lucid test</title><style>p { color: #123456; }</style></head><body><p>Hi</p></body></html>");
        assert_eq!(document_title(&doc).as_deref(), Some("Lucid test"));
        let sheet = collect_styles(&doc);
        assert_eq!(sheet.rules.len(), 1);
        assert_eq!(sheet.rules[0].selector, "p");
    }

    #[test]
    fn produces_paint_operations() {
        let doc = parse_html("<h1>Lucid</h1><p>Independent rendering.</p>");
        let output = layout_document(&doc, &Stylesheet::default(), 900.0);
        assert!(output.height > 0.0);
        assert!(output.operations.iter().any(|op| matches!(op, PaintOp::Text { text, .. } if text == "Lucid")));
    }
}
