//! Paging: an opaque cursor and one page of results.
//!
//! Pattern: Iterator (cursor paging) — the caller hands back `next` until it is `None`. The cursor is
//! opaque to callers; this core encodes an offset, another provider may encode anything.

/// Opaque position in a list (`Cursor` in `common.capnp`; none = start).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cursor(pub String);

impl Cursor {
    /// The cursor for "skip `n` items".
    pub fn at(n: usize) -> Self {
        Self(n.to_string())
    }

    /// The offset this core encoded, or 0 for a cursor it does not understand.
    pub fn offset(&self) -> usize {
        self.0.parse().unwrap_or(0)
    }
}

/// One page of results; `next` is `None` on the last page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page<T> {
    /// The items of this page.
    pub items: Vec<T>,
    /// Where the next page starts.
    pub next: Option<Cursor>,
}

impl<T> Page<T> {
    /// Cut one page of `size` items out of a complete, ordered list.
    pub fn cut(all: Vec<T>, cursor: Option<&Cursor>, size: usize) -> Self {
        let offset = cursor.map_or(0, Cursor::offset);
        let more = all.len() > offset.saturating_add(size);
        let items = all.into_iter().skip(offset).take(size).collect();
        Self {
            items,
            next: more.then(|| Cursor::at(offset.saturating_add(size))),
        }
    }
}
