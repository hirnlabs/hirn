//! Myers block-level diffing between the last projection and an external edit.
//!
//! The diff operates on block text, not characters: the projection is
//! block-granular, so the unit of change is a block. The result feeds the
//! CRDT ops Tier 2 applies (`insert_block`, `delete_block`, `move_block`,
//! text edits).

/// One edit turning the old block list into the new one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockOp {
    /// Insert `text` at `index`.
    Insert { index: usize, text: String },
    /// Delete the block at `index`.
    Delete { index: usize },
    /// Replace the text of the block at `index`.
    Update { index: usize, text: String },
    /// Move the block at `from` to `to`.
    Move { from: usize, to: usize },
}

/// Myers' O((N+M)D) shortest edit script over two block lists.
///
/// Returns `(old_index, new_index)` pairs that are *kept*, i.e. the blocks
/// that need no edit. Everything else is an insert or a delete.
pub fn myers_kept(old: &[&str], new: &[&str]) -> Vec<(usize, usize)> {
    let n = old.len();
    let m = new.len();
    if n == 0 || m == 0 {
        return Vec::new();
    }
    let max = n + m;
    let offset = max as isize;
    let mut v = vec![0isize; 2 * max + 1];
    // `trace[d]` is the frontier before round `d`, so the path can be walked
    // back once the end is reached.
    let mut trace: Vec<Vec<isize>> = Vec::new();

    let mut found = None;
    'search: for d in 0..=max {
        trace.push(v.clone());
        let d = d as isize;
        let mut k = -d;
        while k <= d {
            let index = (offset + k) as usize;
            let mut x = if k == -d || (k != d && v[index - 1] < v[index + 1]) {
                v[index + 1]
            } else {
                v[index - 1] + 1
            };
            let mut y = x - k;
            while x < n as isize && y < m as isize && old[x as usize] == new[y as usize] {
                x += 1;
                y += 1;
            }
            v[index] = x;
            if x >= n as isize && y >= m as isize {
                found = Some(d as usize);
                break 'search;
            }
            k += 2;
        }
    }

    let Some(depth) = found else {
        return Vec::new();
    };
    let mut kept = Vec::new();
    let (mut x, mut y) = (n as isize, m as isize);
    for d in (0..=depth).rev() {
        if d == 0 {
            while x > 0 && y > 0 {
                x -= 1;
                y -= 1;
                kept.push((x as usize, y as usize));
            }
            break;
        }
        let v = &trace[d];
        let d = d as isize;
        let k = x - y;
        let index = (offset + k) as usize;
        let previous_k = if k == -d || (k != d && v[index - 1] < v[index + 1]) {
            k + 1
        } else {
            k - 1
        };
        let previous_x = v[(offset + previous_k) as usize];
        let previous_y = previous_x - previous_k;
        while x > previous_x && y > previous_y && x > 0 && y > 0 {
            x -= 1;
            y -= 1;
            kept.push((x as usize, y as usize));
        }
        x = previous_x;
        y = previous_y;
    }
    kept.reverse();
    kept
}

/// Compute the block-level edit script.
///
/// A block whose text reappears at another position is reported as a move; an
/// unmatched old block paired with an unmatched new block at the same point in
/// the script is an update rather than a delete plus an insert.
pub fn diff_blocks(old: &[String], new: &[String]) -> Vec<BlockOp> {
    let old_text: Vec<&str> = old.iter().map(String::as_str).collect();
    let new_text: Vec<&str> = new.iter().map(String::as_str).collect();
    let kept = myers_kept(&old_text, &new_text);

    let kept_old: std::collections::HashSet<usize> =
        kept.iter().map(|(old_index, _)| *old_index).collect();
    let kept_new: std::collections::HashSet<usize> =
        kept.iter().map(|(_, new_index)| *new_index).collect();
    let mut deletions: Vec<usize> = (0..old_text.len())
        .filter(|index| !kept_old.contains(index))
        .collect();
    let insertions: Vec<usize> = (0..new_text.len())
        .filter(|index| !kept_new.contains(index))
        .collect();

    let mut ops = Vec::new();
    let mut pending_inserts = Vec::new();
    // Pair a deletion with an insert of the same text: that is a move.
    for new_index in insertions {
        let candidate = deletions
            .iter()
            .position(|old_index| old_text[*old_index] == new_text[new_index]);
        match candidate {
            Some(position) => {
                let from = deletions.remove(position);
                ops.push(BlockOp::Move {
                    from,
                    to: new_index,
                });
            }
            None => pending_inserts.push(new_index),
        }
    }

    // What remains is a replacement, a pure delete, or a pure insert. Pair
    // leftovers in order so an in-place text change is one Update.
    let paired = deletions.len().min(pending_inserts.len());
    for new_index in pending_inserts.iter().take(paired) {
        ops.push(BlockOp::Update {
            index: *new_index,
            text: new_text[*new_index].to_string(),
        });
    }
    for old_index in deletions.into_iter().skip(paired) {
        ops.push(BlockOp::Delete { index: old_index });
    }
    for new_index in pending_inserts.into_iter().skip(paired) {
        ops.push(BlockOp::Insert {
            index: new_index,
            text: new_text[new_index].to_string(),
        });
    }
    ops
}

/// Whether the diff is empty, i.e. the projection and the file agree.
pub fn is_noop(ops: &[BlockOp]) -> bool {
    ops.is_empty()
}
