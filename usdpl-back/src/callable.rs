use usdpl_core::serdes::Primitive;

pub trait Callable: Send + Sync {
    fn call(&mut self, params: Vec<Primitive>) -> Vec<Primitive>;
}
