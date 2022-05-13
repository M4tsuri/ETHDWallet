use alloc::collections::VecDeque;
use core::task::{Waker, Context, RawWaker, RawWakerVTable, Poll};

use super::Task;

pub struct Executor<T: Task> {
    task_queue: VecDeque<T>
}

impl<T: Task> Executor<T> {
    pub fn new() -> Self {
        Self { task_queue: VecDeque::new() }
    }

    pub fn spawn(&mut self, task: T) {
        self.task_queue.push_back(task)
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);

            match task.poll(&mut context) {
                Poll::Pending => self.task_queue.push_back(task),
                Poll::Ready(_) => {}
            }
        }
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn nop(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    RawWaker::new(
        0 as *const (),
        &RawWakerVTable::new(clone, nop, nop, nop)
    )
} 

fn dummy_waker() -> Waker {
    unsafe {
        Waker::from_raw(dummy_raw_waker())
    }
}


