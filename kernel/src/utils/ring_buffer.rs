use crate::Mutex;

pub struct RingBuffer<T, const N: usize> {
    inner: Mutex<RingBufferInner<T, N>>,
}

impl<T, const N: usize> RingBuffer<T, N> {
    pub fn new() -> Self {
        RingBuffer {
            inner: Mutex::new(RingBufferInner {
                buffer: [const { None }; N],
                head: 0,
                tail: 0,
            }),
        }
    }

    pub fn push(&self, value: T) -> Result<usize, ()> {
        let mut inner = self.inner.lock();
        let head = inner.head;
        if (head + 1) % N == inner.tail {
            return Err(());
        }
        inner.buffer[head] = Some(value);
        inner.head = (head + 1) % N;
        Ok(head)
    }

    pub fn pop(&self) -> Option<T> {
        let mut inner = self.inner.lock();
        let tail = inner.tail;
        if tail == inner.head {
            return None;
        }
        let value = inner.buffer[tail].take();
        inner.tail = (tail + 1) % N;
        value
    }

    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock();
        inner.head == inner.tail
    }
}

struct RingBufferInner<T, const N: usize> {
    buffer: [Option<T>; N],
    head: usize,
    tail: usize,
}
