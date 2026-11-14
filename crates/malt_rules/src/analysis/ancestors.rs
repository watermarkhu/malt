//! # Ancestor predicates, memoised per node
//!
//! `Node::parent` in tree-sitter walks down from the root, so one call costs
//! O(depth). A predicate that climbs to the root for every node therefore costs
//! O(depth²) per node, which a deeply nested file makes slow. [`has_ancestor`]
//! remembers the answer for every node it climbs past, so each node's parent is
//! looked up once per parse.

use std::cell::RefCell;
use std::collections::HashMap;

use tree_sitter::Node;

/// The memo of [`has_ancestor`]: one table per predicate slot, valid for one
/// parse epoch (see `malt_core::parse_epoch`), because node ids are addresses
/// that a later parse can reuse.
#[derive(Default)]
struct AncestorMemo {
    epoch: u64,
    /// For each slot, the node id to "this node or one of its ancestors matches".
    slots: [HashMap<usize, bool>; SLOTS],
}

/// The number of predicates that can be memoised.
const SLOTS: usize = 1;

/// Slot of `SymbolTable`-free `syntax_errors` `inside_error`: an ERROR ancestor.
pub(crate) const SLOT_ERROR: usize = 0;

thread_local! {
    static MEMO: RefCell<AncestorMemo> = RefCell::new(AncestorMemo::default());
}

/// Whether a strict ancestor of `node` matches `matches`. `slot` names the
/// predicate, so each predicate has its own memo: a predicate must always be
/// passed with the same slot and the same `matches`.
pub(crate) fn has_ancestor(node: Node, slot: usize, matches: fn(Node) -> bool) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };
    let epoch = malt_core::parse_epoch();
    MEMO.with(|memo| {
        let mut memo = memo.borrow_mut();
        if memo.epoch != epoch {
            memo.slots = Default::default();
            memo.epoch = epoch;
        }
        let table = &mut memo.slots[slot];
        // Climb until a node with a known answer, or the root. `value` is the
        // answer for the node above the climbed path.
        let mut path = Vec::new();
        let mut value = false;
        let mut current = Some(parent);
        while let Some(n) = current {
            if let Some(&known) = table.get(&n.id()) {
                value = known;
                break;
            }
            path.push(n);
            current = n.parent();
        }
        // The answer for a node is its own match or the answer above it.
        let mut answer = value;
        for n in path.iter().rev() {
            answer |= matches(*n);
            table.insert(n.id(), answer);
        }
        answer
    })
}

/// The ancestors of `node` below `root`, the root first and `node` last.
///
/// Walks down once, the way `Node::parent` does for every single step, so
/// climbing a path of length `d` costs one descent instead of `d`.
pub(crate) fn path_from<'t>(root: Node<'t>, node: Node<'t>) -> Vec<Node<'t>> {
    let mut path = vec![root];
    let mut current = root;
    while current.id() != node.id() {
        match current.child_with_descendant(node) {
            Some(next) => {
                path.push(next);
                current = next;
            }
            None => break,
        }
    }
    path
}
