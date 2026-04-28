//! Tiny fixed-size operation queue for "async" LCD updates.
//!
//! This is a minimal ring buffer used by the LCD driver to enqueue operations without `std`:
//! - `Cmd(u8)`   — send a command byte (RS=0)
//! - `Data(u8)`  — send a data byte (RS=1)
//!
//! The main LCD driver calls `pop()` from `update(now_ms, i2c)` and executes at most
//! one queued operation per call when `now_ms >= next_ms`.


#[derive(Copy, Clone)]
/// A single queued LCD operation for async-style updates.
pub enum Op {
    Cmd(u8),
    Data(u8),
}

/// Queue capacity (number of operations).
pub const QN: usize = 64;

/// Fixed-size ring buffer for LCD operations.
///
/// This queue is `no_std` friendly and uses `Option<Op>` slots.
/// `push()` returns `false` if the queue is full.

pub struct OpQueue {
    buf: [Option<Op>; QN],
    head: usize,
    tail: usize,
}

impl OpQueue {
    /// Create an empty queue.
    pub(crate) const fn new() -> Self {
        Self { buf: [None; QN], head: 0, tail: 0 }
    }
    /// Push an operation. Returns false if the queue is full.
    pub(crate) fn push(&mut self, op: Op) -> bool {
        let next = (self.tail + 1) % QN;
        if next == self.head { return false; } // full
        self.buf[self.tail] = Some(op);
        self.tail = next;
        true
    }

    /// Pop the next operation (FIFO). Returns None if empty.
    pub(crate) fn pop(&mut self) -> Option<Op> {
        if self.head == self.tail { return None; } // empty
        let op = self.buf[self.head].take();
        self.head = (self.head + 1) % QN;
        op
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub(crate) fn clear(&mut self) {
        self.buf = [None; QN];
        self.head = 0;
        self.tail = 0;
    }
}
