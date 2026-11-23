use std::cell::RefCell;
use std::rc::{Rc, Weak};

pub type NodeRef = Rc<RefCell<Node>>;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Document,
    Element { tag_name: String, attributes: Vec<(String, String)> },
    Text(String),
    Comment(String),
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub parent: Option<Weak<RefCell<Node>>>,
    pub children: Vec<NodeRef>,
}

impl Node {
    pub fn new(kind: NodeKind) -> NodeRef {
        Rc::new(RefCell::new(Self { kind, parent: None, children: Vec::new() }))
    }

    pub fn append(parent: &NodeRef, child: NodeRef) {
        child.borrow_mut().parent = Some(Rc::downgrade(parent));
        parent.borrow_mut().children.push(child);
    }

    pub fn text_content(node: &NodeRef) -> String {
        let borrowed = node.borrow();
        match &borrowed.kind {
            NodeKind::Text(text) => text.clone(),
            _ => borrowed.children.iter().map(Self::text_content).collect::<Vec<_>>().join(""),
        }
    }
}

#[derive(Debug)]
pub struct Document {
    pub root: NodeRef,
}

impl Default for Document {
    fn default() -> Self {
        Self { root: Node::new(NodeKind::Document) }
    }
}

impl Document {
    pub fn title_text(&self) -> Option<String> {
        fn find(node: &NodeRef) -> Option<String> {
            let n = node.borrow();
            if matches!(&n.kind, NodeKind::Element { tag_name, .. } if tag_name == "title") {
                return Some(Node::text_content(node).trim().to_string());
            }
            for child in &n.children {
                if let Some(value) = find(child) { return Some(value); }
            }
            None
        }
        find(&self.root)
    }
}
