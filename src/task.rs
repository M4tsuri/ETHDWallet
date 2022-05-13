use core::{future::Future, pin::Pin, task::{Context, Poll}};
use alloc::boxed::Box;

pub mod executor;

pub trait Task {
    type Output;

    fn poll(&mut self, context: &mut Context) -> Poll<Self::Output>;
}

pub struct ETask<O> {
    future: Pin<Box<dyn Future<Output = O>>>
}

impl<O> ETask<O> {
    pub fn new(future: impl Future<Output = O> + 'static) -> Self {
        Self { future: Box::pin(future) }
    }
}

impl<O> Task for ETask<O> {
    type Output = O;

    fn poll(&mut self, context: &mut Context) -> Poll<Self::Output> {
        self.future.as_mut().poll(context)
    }
}

