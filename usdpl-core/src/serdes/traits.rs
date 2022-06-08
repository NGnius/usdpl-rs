/// Load an object from the buffer
pub trait Loadable: Sized {
    /// Read the buffer, building the object and returning the amount of bytes read.
    /// If anything is wrong with the buffer, None should be returned.
    fn load(buffer: &[u8]) -> (Option<Self>, usize);
}

/// Dump an object into the buffer
pub trait Dumpable {
    /// Write the object to the buffer, returning the amount of bytes written.
    /// If anything is wrong, false should be returned.
    fn dump(&self, buffer: &mut [u8]) -> (bool, usize);
}
