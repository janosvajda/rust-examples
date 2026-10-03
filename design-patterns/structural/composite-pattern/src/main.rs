// Composite pattern: files and folders. A folder can contain files and other
// folders, but you can ask a folder the same questions you ask a file (how
// big are you? print yourself), and it answers by asking its contents.

/// One node in the tree: either a single file (a leaf) or a folder that
/// contains more nodes (a composite).
#[derive(Debug)]
enum Node {
    File { name: String, size_kb: u64 },
    Folder { name: String, children: Vec<Node> },
}

impl Node {
    fn file(name: &str, size_kb: u64) -> Node {
        Node::File {
            name: name.to_string(),
            size_kb,
        }
    }

    fn folder(name: &str, children: Vec<Node>) -> Node {
        Node::Folder {
            name: name.to_string(),
            children,
        }
    }

    fn name(&self) -> &str {
        match self {
            Node::File { name, .. } | Node::Folder { name, .. } => name,
        }
    }

    /// The key idea: the same question works on both kinds of node.
    /// A file knows its own size; a folder adds up the sizes of its children,
    /// which may themselves be folders, so this recurses down the tree.
    fn size_kb(&self) -> u64 {
        match self {
            Node::File { size_kb, .. } => *size_kb,
            Node::Folder { children, .. } => children.iter().map(Node::size_kb).sum(),
        }
    }

    /// Counts files only, at any depth.
    fn file_count(&self) -> usize {
        match self {
            Node::File { .. } => 1,
            Node::Folder { children, .. } => children.iter().map(Node::file_count).sum(),
        }
    }

    /// Finds every file whose name ends with `extension`, with its full path.
    fn find_by_extension(&self, extension: &str) -> Vec<String> {
        let mut found = Vec::new();
        self.collect_matches(extension, "", &mut found);
        found
    }

    fn collect_matches(&self, extension: &str, parent_path: &str, found: &mut Vec<String>) {
        let path = format!("{parent_path}/{}", self.name());
        match self {
            Node::File { name, .. } => {
                if name.ends_with(extension) {
                    found.push(path);
                }
            }
            Node::Folder { children, .. } => {
                for child in children {
                    child.collect_matches(extension, &path, found);
                }
            }
        }
    }

    /// Prints the tree with indentation, like the `tree` command.
    fn print(&self, depth: usize) {
        let indent = "    ".repeat(depth);
        match self {
            Node::File { name, size_kb } => println!("{indent}{name} ({size_kb} KB)"),
            Node::Folder { name, children } => {
                println!("{indent}{name}/ ({} KB total)", self.size_kb());
                for child in children {
                    child.print(depth + 1);
                }
            }
        }
    }
}

fn main() {
    let home = Node::folder(
        "home",
        vec![
            Node::file("notes.txt", 4),
            Node::folder(
                "photos",
                vec![
                    Node::file("beach.jpg", 2_400),
                    Node::file("mountains.jpg", 3_100),
                    Node::folder("2024", vec![Node::file("birthday.jpg", 1_800)]),
                ],
            ),
            Node::folder(
                "projects",
                vec![
                    Node::folder(
                        "rust-examples",
                        vec![Node::file("main.rs", 12), Node::file("README.md", 6)],
                    ),
                    Node::folder("empty", vec![]),
                ],
            ),
        ],
    );

    home.print(0);

    println!("\nTotal: {} KB in {} files", home.size_kb(), home.file_count());
    println!("All .jpg files: {:?}", home.find_by_extension(".jpg"));

    // A single file answers the same questions as the whole tree.
    let single = Node::file("todo.md", 1);
    println!("A lone file: {} KB, {} file", single.size_kb(), single.file_count());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Node {
        Node::folder(
            "root",
            vec![
                Node::file("a.txt", 10),
                Node::folder("sub", vec![Node::file("b.jpg", 20), Node::file("c.jpg", 30)]),
                Node::folder("empty", vec![]),
            ],
        )
    }

    #[test]
    fn a_file_reports_its_own_size() {
        assert_eq!(Node::file("x", 7).size_kb(), 7);
        assert_eq!(Node::file("x", 7).file_count(), 1);
    }

    #[test]
    fn a_folder_adds_up_everything_inside() {
        assert_eq!(sample().size_kb(), 60);
        assert_eq!(sample().file_count(), 3);
    }

    #[test]
    fn an_empty_folder_is_zero() {
        let empty = Node::folder("empty", vec![]);
        assert_eq!(empty.size_kb(), 0);
        assert_eq!(empty.file_count(), 0);
    }

    #[test]
    fn search_goes_through_every_level() {
        assert_eq!(
            sample().find_by_extension(".jpg"),
            vec!["/root/sub/b.jpg", "/root/sub/c.jpg"]
        );
        assert!(sample().find_by_extension(".pdf").is_empty());
    }
}
