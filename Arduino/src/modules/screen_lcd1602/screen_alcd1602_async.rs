#[derive(Copy, Clone)]
pub enum Op {
    Cmd(u8),
    Data(u8),

    WaitMs(u16),
}

pub const QN: usize = 32;

pub struct OpQueue {
    buf: [Option<Op>; QN],
    head: usize,
    tail: usize,
}

impl OpQueue {
    pub(crate) const fn new() -> Self {
        Self { buf: [None; QN], head: 0, tail: 0 }
    }

    pub(crate) fn push(&mut self, op: Op) -> bool {
        let next = (self.tail + 1) % QN;
        if next == self.head { return false; } // full
        self.buf[self.tail] = Some(op);
        self.tail = next;
        true
    }

    pub(crate) fn pop(&mut self) -> Option<Op> {
        if self.head == self.tail { return None; } // empty
        let op = self.buf[self.head].take();
        self.head = (self.head + 1) % QN;
        op
    }

    fn is_empty(&self) -> bool {
        self.head == self.tail
    }
}