use crate::dom::{Document, Node, NodeKind};

/// Parse a deliberately limited HTML subset into Lucid's own DOM tree.
///
/// This prototype recognizes start/end tags, quoted or unquoted attributes and text.
/// It is NOT the HTML Living Standard tree-construction algorithm yet. Malformed-markup
/// recovery, entities, raw-text states, templates, namespaces and scripting are pending.
pub fn parse_document(input: &str) -> Document {
    let mut document = Document::default();
    let mut stack = vec![document.root.clone()];
    let mut i = 0;
    let bytes = input.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'<' {
            if i + 3 < bytes.len() && &bytes[i..i + 4] == b"<!--" {
                let start = i + 4;
                let end = input[start..].find("-->").map(|n| start + n).unwrap_or(bytes.len());
                Node::append(stack.last().unwrap(), Node::new(NodeKind::Comment(input[start..end].to_string())));
                i = (end + 3).min(bytes.len());
                continue;
            }

            if let Some(relative_end) = input[i..].find('>') {
                let end = i + relative_end;
                let token = input[i + 1..end].trim();
                if let Some(name) = token.strip_prefix('/') {
                    let name = name.trim().to_ascii_lowercase();
                    if stack.len() > 1 {
                        let position = stack.iter().rposition(|node| matches!(&node.borrow().kind, NodeKind::Element { tag_name, .. } if tag_name == &name));
                        if let Some(position) = position { stack.truncate(position); }
                    }
                } else if !token.is_empty() && !token.starts_with('!') && !token.starts_with('?') {
                    let self_closing = token.ends_with('/');
                    let token = token.trim_end_matches('/').trim();
                    let split = token.find(char::is_whitespace).unwrap_or(token.len());
                    let tag = token[..split].to_ascii_lowercase();
                    let attrs = parse_attributes(token.get(split..).unwrap_or(""));
                    let node = Node::new(NodeKind::Element { tag_name: tag.clone(), attributes: attrs });
                    Node::append(stack.last().unwrap(), node.clone());
                    let void = matches!(tag.as_str(), "area"|"base"|"br"|"col"|"embed"|"hr"|"img"|"input"|"link"|"meta"|"param"|"source"|"track"|"wbr");
                    if !self_closing && !void { stack.push(node); }
                }
                i = end + 1;
                continue;
            }
        }

        let start = i;
        while i < bytes.len() && bytes[i] != b'<' { i += 1; }
        if i > start {
            let text = String::from_utf8_lossy(&bytes[start..i]).into_owned();
            if !text.is_empty() { Node::append(stack.last().unwrap(), Node::new(NodeKind::Text(text))); }
        }
    }
    document
}

fn parse_attributes(source: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut chars = source.trim().chars().peekable();
    while chars.peek().is_some() {
        while chars.peek().is_some_and(|c| c.is_whitespace()) { chars.next(); }
        if chars.peek().is_none() { break; }
        let mut name = String::new();
        while chars.peek().is_some_and(|c| !c.is_whitespace() && *c != '=') { name.push(chars.next().unwrap()); }
        while chars.peek().is_some_and(|c| c.is_whitespace()) { chars.next(); }
        let mut value = String::new();
        if chars.peek() == Some(&'=') {
            chars.next();
            while chars.peek().is_some_and(|c| c.is_whitespace()) { chars.next(); }
            if let Some(quote @ ('"' | '\'')) = chars.peek().copied() {
                chars.next();
                while let Some(c) = chars.next() { if c == quote { break; } value.push(c); }
            } else {
                while chars.peek().is_some_and(|c| !c.is_whitespace()) { value.push(chars.next().unwrap()); }
            }
        }
        if !name.is_empty() { result.push((name.to_ascii_lowercase(), value)); }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::NodeKind;

    #[test]
    fn builds_basic_document_tree_and_title() {
        let doc = parse_document("<html><head><title>Lucid</title></head><body><p>Hello</p></body></html>");
        assert_eq!(doc.title_text().as_deref(), Some("Lucid"));
        assert!(matches!(&doc.root.borrow().children[0].borrow().kind, NodeKind::Element { tag_name, .. } if tag_name == "html"));
    }

    #[test]
    fn parses_attributes() {
        let doc = parse_document("<a href='https://example.com' target=_blank>Open</a>");
        let html = &doc.root.borrow().children[0];
        let attrs = match &html.borrow().kind { NodeKind::Element { attributes, .. } => attributes.clone(), _ => vec![] };
        assert_eq!(attrs.len(), 2);
    }
}
