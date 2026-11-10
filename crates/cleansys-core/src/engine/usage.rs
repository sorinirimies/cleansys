//! "Where did my disk go?" — a size tree of one folder, biggest entries first.
//!
//! Read-only and bounded like the other walks: symlinks are not followed, a drive mounted
//! below the start folder is not entered, and the walk stops after `MAX_VISITED_DIRS`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use super::exec::{device_of_path, same_device};

const MAX_VISITED_DIRS: usize = 1_000_000;

/// One folder or file with its total size. `children` holds the biggest sub-entries (down
/// to the requested depth); `hidden` is what the cut-off ones add up to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Node {
    pub path: PathBuf,
    pub bytes: u64,
    pub is_dir: bool,
    pub children: Vec<Node>,
    /// Bytes in entries not listed in `children`.
    pub hidden: u64,
}

/// Size tree of `root`: `depth` levels of children, at most `top` per folder.
pub fn disk_usage(root: &Path, depth: usize, top: usize) -> Node {
    let visited = AtomicUsize::new(0);
    walk(root, depth, top.max(1), &visited)
}

fn walk(path: &Path, depth: usize, top: usize, visited: &AtomicUsize) -> Node {
    use rayon::prelude::*;
    let Ok(meta) = fs::symlink_metadata(path) else {
        return Node {
            path: path.to_path_buf(),
            ..Node::default()
        };
    };
    if !meta.is_dir() || meta.file_type().is_symlink() {
        return Node {
            path: path.to_path_buf(),
            bytes: if meta.file_type().is_symlink() {
                0
            } else {
                disk_bytes(&meta)
            },
            ..Node::default()
        };
    }
    let mut node = Node {
        path: path.to_path_buf(),
        is_dir: true,
        ..Node::default()
    };
    if visited.fetch_add(1, Ordering::Relaxed) > MAX_VISITED_DIRS {
        return node;
    }
    let here = device_of_path(path);
    let Ok(rd) = fs::read_dir(path) else {
        return node;
    };
    let entries: Vec<fs::DirEntry> = rd
        .flatten()
        .filter(|e| e.file_type().map(|t| !t.is_dir()).unwrap_or(true) || same_device(here, e))
        .collect();
    let mut kids: Vec<Node> = entries
        .par_iter()
        .map(|e| walk(&e.path(), depth.saturating_sub(1), top, visited))
        .collect();
    node.bytes = kids.iter().map(|k| k.bytes).sum();
    kids.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.path.cmp(&b.path)));
    if depth > 0 {
        let rest = kids.split_off(top.min(kids.len()));
        node.hidden = rest.iter().map(|k| k.bytes).sum();
        node.children = kids;
    }
    node
}

/// Space the file really occupies (a sparse VM disk is not as big as its length).
#[cfg(unix)]
fn disk_bytes(m: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    m.blocks().saturating_mul(512)
}

#[cfg(not(unix))]
fn disk_bytes(m: &fs::Metadata) -> u64 {
    m.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(p: &Path, n: usize) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, vec![b'x'; n]).unwrap();
    }

    #[test]
    fn sums_folders_and_lists_the_biggest_first() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        // sizes are on-disk (block) sizes, so use clearly different magnitudes
        write(&r.join("big/a"), 2_000_000);
        write(&r.join("big/b"), 1_000_000);
        write(&r.join("small/c"), 100_000);
        write(&r.join("tiny"), 5);
        let n = disk_usage(r, 1, 2);
        assert_eq!(n.children.len(), 2);
        assert!(n.children[0].path.ends_with("big"));
        assert!(n.children[1].path.ends_with("small"));
        assert!(n.children[0].bytes >= 3_000_000);
        assert!(n.hidden > 0, "the cut-off entry is accounted for");
        let listed: u64 = n.children.iter().map(|c| c.bytes).sum();
        assert_eq!(n.bytes, listed + n.hidden);
        assert!(
            n.children[0].children.is_empty(),
            "depth 1: no grandchildren"
        );
    }
}
