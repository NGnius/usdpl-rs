use usdpl_core::serdes::Primitive;

/// A function which can be called from the front-end (remotely)
pub trait Callable: Send + Sync {
    /// Invoke the function
    fn call(&mut self, params: Vec<Primitive>) -> Vec<Primitive>;
}

impl<F: (FnMut(Vec<Primitive>) -> Vec<Primitive>) + Send + Sync> Callable for F {
    fn call(&mut self, params: Vec<Primitive>) -> Vec<Primitive> {
        (self)(params)
    }
}
