use std::sync::{Arc, Mutex};

use usdpl_core::serdes::Primitive;

/// A mutable function which can be called from the front-end (remotely)
pub trait MutCallable: Send + Sync {
    /// Invoke the function
    fn call(&mut self, params: Vec<Primitive>) -> Vec<Primitive>;
}

impl<F: (FnMut(Vec<Primitive>) -> Vec<Primitive>) + Send + Sync> MutCallable for F {
    fn call(&mut self, params: Vec<Primitive>) -> Vec<Primitive> {
        (self)(params)
    }
}

/// A function which can be called from the front-end (remotely)
pub trait Callable: Send + Sync {
    /// Invoke the function
    fn call(&self, params: Vec<Primitive>) -> Vec<Primitive>;
}

impl<F: (Fn(Vec<Primitive>) -> Vec<Primitive>) + Send + Sync> Callable for F {
    fn call(&self, params: Vec<Primitive>) -> Vec<Primitive> {
        (self)(params)
    }
}

/// An async function which can be called from the front-end (remotely)
#[async_trait::async_trait]
pub trait AsyncCallable: Send + Sync {
    /// Invoke the function
    async fn call(&self, params: Vec<Primitive>) -> Vec<Primitive>;
}

#[async_trait::async_trait]
impl<F: (Fn(Vec<Primitive>) -> A) + Send + Sync, A: core::future::Future<Output=Vec<Primitive>> + Send> AsyncCallable for F {
    async fn call(&self, params: Vec<Primitive>) -> Vec<Primitive> {
        (self)(params).await
    }
}

pub enum WrappedCallable {
    Blocking(Arc<Mutex<Box<dyn MutCallable>>>),
    Ref(Arc<Box<dyn Callable>>),
    Async(Arc<Box<dyn AsyncCallable>>),
}

impl WrappedCallable {
    pub fn new_ref<T: Callable + 'static>(callable: T) -> Self {
        Self::Ref(Arc::new(Box::new(callable)))
    }

    pub fn new_locking<T: MutCallable + 'static>(callable: T) -> Self {
        Self::Blocking(Arc::new(Mutex::new(Box::new(callable))))
    }

    pub fn new_async<T: AsyncCallable + 'static>(callable: T) -> Self {
        Self::Async(Arc::new(Box::new(callable)))
    }
}

impl Clone for WrappedCallable {
    fn clone(&self) -> Self {
        match self {
            Self::Blocking(x) => Self::Blocking(x.clone()),
            Self::Ref(x) => Self::Ref(x.clone()),
            Self::Async(x) => Self::Async(x.clone()),
        }
    }
}

#[async_trait::async_trait]
impl AsyncCallable for WrappedCallable {
    async fn call(&self, params: Vec<Primitive>) -> Vec<Primitive> {
        match self {
            Self::Blocking(mut_callable) => {
                mut_callable
                    .lock()
                    .expect("Failed to acquire mut_callable lock")
                    .call(params)
            },
            Self::Ref(callable) => callable.call(params),
            Self::Async(async_callable) => async_callable.call(params).await,
        }
    }
}
