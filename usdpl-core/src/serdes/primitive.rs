use super::{DumpError, Dumpable, LoadError, Loadable};

/// Primitive types supported for communication between the USDPL back- and front-end.
/// These are used for sending over the TCP connection.
pub enum Primitive {
    /// Null or unsupported object
    Empty,
    /// String-like
    String(String),
    /// f32
    F32(f32),
    /// f64
    F64(f64),
    /// u32
    U32(u32),
    /// u64
    U64(u64),
    /// i32
    I32(i32),
    /// i64
    I64(i64),
    /// boolean
    Bool(bool),
    /// Non-primitive in Json format
    Json(String),
}

impl Primitive {
    /// Discriminant -- first byte of a dumped primitive
    const fn discriminant(&self) -> u8 {
        match self {
            Self::Empty => 1,
            Self::String(_) => 2,
            Self::F32(_) => 3,
            Self::F64(_) => 4,
            Self::U32(_) => 5,
            Self::U64(_) => 6,
            Self::I32(_) => 7,
            Self::I64(_) => 8,
            Self::Bool(_) => 9,
            Self::Json(_) => 10,
        }
    }
}

impl Loadable for Primitive {
    fn load(buf: &[u8]) -> Result<(Self, usize), LoadError> {
        if buf.len() == 0 {
            return Err(LoadError::TooSmallBuffer);
        }
        let mut result: (Self, usize) = match buf[0] {
            //0 => (None, 0),
            1 => (Self::Empty, 0),
            2 => String::load(&buf[1..]).map(|(obj, len)| (Self::String(obj), len))?,
            3 => f32::load(&buf[1..]).map(|(obj, len)| (Self::F32(obj), len))?,
            4 => f64::load(&buf[1..]).map(|(obj, len)| (Self::F64(obj), len))?,
            5 => u32::load(&buf[1..]).map(|(obj, len)| (Self::U32(obj), len))?,
            6 => u64::load(&buf[1..]).map(|(obj, len)| (Self::U64(obj), len))?,
            7 => i32::load(&buf[1..]).map(|(obj, len)| (Self::I32(obj), len))?,
            8 => i64::load(&buf[1..]).map(|(obj, len)| (Self::I64(obj), len))?,
            9 => bool::load(&buf[1..]).map(|(obj, len)| (Self::Bool(obj), len))?,
            10 => String::load(&buf[1..]).map(|(obj, len)| (Self::Json(obj), len))?,
            _ => return Err(LoadError::InvalidData),
        };
        result.1 += 1;
        Ok(result)
    }
}

impl Dumpable for Primitive {
    fn dump(&self, buf: &mut [u8]) -> Result<usize, DumpError> {
        if buf.len() == 0 {
            return Err(DumpError::TooSmallBuffer);
        }
        buf[0] = self.discriminant();
        let mut result = match self {
            Self::Empty => Ok(0),
            Self::String(s) => s.dump(&mut buf[1..]),
            Self::F32(x) => x.dump(&mut buf[1..]),
            Self::F64(x) => x.dump(&mut buf[1..]),
            Self::U32(x) => x.dump(&mut buf[1..]),
            Self::U64(x) => x.dump(&mut buf[1..]),
            Self::I32(x) => x.dump(&mut buf[1..]),
            Self::I64(x) => x.dump(&mut buf[1..]),
            Self::Bool(x) => x.dump(&mut buf[1..]),
            Self::Json(x) => x.dump(&mut buf[1..]),
        }?;
        result += 1;
        Ok(result)
    }
}

impl std::convert::Into<Primitive> for &str {
    fn into(self) -> Primitive {
        Primitive::String(self.to_string())
    }
}

impl std::convert::Into<Primitive> for () {
    fn into(self) -> Primitive {
        Primitive::Empty
    }
}

macro_rules! into_impl {
    ($type:ty, $variant:ident) => {
        impl std::convert::Into<Primitive> for $type {
            fn into(self) -> Primitive {
                Primitive::$variant(self)
            }
        }
    }
}

into_impl! {String, String}
into_impl! {bool, Bool}

into_impl! {u32, U32}
into_impl! {u64, U64}

into_impl! {i32, I32}
into_impl! {i64, I64}

into_impl! {f32, F32}
into_impl! {f64, F64}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_idempotence_test() {
        let data = "Test";
        let primitive = Primitive::String(data.to_string());
        let mut buffer = [0u8; 128];
        let write_len = primitive.dump(&mut buffer).expect("Dump not ok");
        let (obj, read_len) = Primitive::load(&buffer).expect("Load not ok");
        assert_eq!(
            write_len, read_len,
            "Amount written and amount read do not match"
        );
        if let Primitive::String(result) = obj {
            assert_eq!(data, result, "Data written and read does not match");
        } else {
            panic!("Read non-string primitive");
        }
    }

    #[test]
    fn empty_idempotence_test() {
        let primitive = Primitive::Empty;
        let mut buffer = [0u8; 128];
        let write_len = primitive.dump(&mut buffer).expect("Dump not ok");
        let (obj, read_len) = Primitive::load(&buffer).expect("Load not ok");
        assert_eq!(
            write_len, read_len,
            "Amount written and amount read do not match"
        );
        if let Primitive::Empty = obj {
            //assert_eq!(data, result, "Data written and read does not match");
        } else {
            panic!("Read non-string primitive");
        }
    }
}
