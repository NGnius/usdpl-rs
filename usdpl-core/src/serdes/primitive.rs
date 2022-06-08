use super::{Loadable, Dumpable};

pub enum Primitive {
    Empty,
    String(String),
    F32(f32),
    F64(f64),
    U32(u32),
    U64(u64),
    I32(i32),
    I64(i64),
    Bool(bool),
    Json(String),
}

impl Primitive {
    const fn discriminant(&self) -> u8 {
        match self {
            Self::Empty => 1,
            Self::String(_) => 2,
            Self::F32(_)=> 3,
            Self::F64(_)=> 4,
            Self::U32(_)=> 5,
            Self::U64(_)=> 6,
            Self::I32(_)=> 7,
            Self::I64(_)=> 8,
            Self::Bool(_) => 9,
            Self::Json(_) => 10,
        }
    }
}

impl Loadable for Primitive {
    fn load(buf: &[u8]) -> (Option<Self>, usize) {
        if buf.len() == 0 {
            return (None, 1);
        }
        let mut result: (Option<Self>, usize) = match buf[0] {
            //0 => (None, 0),
            1 => (Some(Self::Empty), 0),
            2 => {
                let (obj, len) = String::load(&buf[1..]);
                (obj.map(Self::String), len)
            },
            3 => {
                let (obj, len) = f32::load(&buf[1..]);
                (obj.map(Self::F32), len)
            },
            4 => {
                let (obj, len) = f64::load(&buf[1..]);
                (obj.map(Self::F64), len)
            },
            5 => {
                let (obj, len) = u32::load(&buf[1..]);
                (obj.map(Self::U32), len)
            },
            6 => {
                let (obj, len) = u64::load(&buf[1..]);
                (obj.map(Self::U64), len)
            },
            7 => {
                let (obj, len) = i32::load(&buf[1..]);
                (obj.map(Self::I32), len)
            },
            8 => {
                let (obj, len) = i64::load(&buf[1..]);
                (obj.map(Self::I64), len)
            },
            9 => {
                let (obj, len) = bool::load(&buf[1..]);
                (obj.map(Self::Bool), len)
            },
            10 => {
                let (obj, len) = String::load(&buf[1..]);
                (obj.map(Self::Json), len)
            }
            _ => (None, 0)
        };
        result.1 += 1;
        result
    }
}


impl Dumpable for Primitive {
    fn dump(&self, buf: &mut [u8]) -> (bool, usize) {
        if buf.len() == 0 {
            return (false, 0);
        }
        buf[0] = self.discriminant();
        let mut result = match self {
            Self::Empty => (true, 0),
            Self::String(s) => s.dump(&mut buf[1..]),
            Self::F32(x)=> x.dump(&mut buf[1..]),
            Self::F64(x)=> x.dump(&mut buf[1..]),
            Self::U32(x)=> x.dump(&mut buf[1..]),
            Self::U64(x)=> x.dump(&mut buf[1..]),
            Self::I32(x)=> x.dump(&mut buf[1..]),
            Self::I64(x)=> x.dump(&mut buf[1..]),
            Self::Bool(x)=> x.dump(&mut buf[1..]),
            Self::Json(x) => x.dump(&mut buf[1..]),
        };
        result.1 += 1;
        result
    }
}

impl std::convert::Into<Primitive> for String {
    fn into(self) -> Primitive {
        Primitive::String(self)
    }
}

impl std::convert::Into<Primitive> for () {
    fn into(self) -> Primitive {
        Primitive::Empty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_idempotence_test() {
        let data = "Test";
        let primitive = Primitive::String(data.to_string());
        let mut buffer = [0u8; 128];
        let (ok, write_len) = primitive.dump(&mut buffer);
        assert!(ok, "Dump not ok");
        let (obj, read_len) = Primitive::load(&buffer);
        assert_eq!(write_len, read_len, "Amount written and amount read do not match");
        assert!(obj.is_some(), "Load not ok");
        if let Some(Primitive::String(result)) = obj {
            assert_eq!(data, result, "Data written and read does not match");
        } else {
            panic!("Read non-string primitive");
        }
    }

    #[test]
    fn empty_idempotence_test() {
        let primitive = Primitive::Empty;
        let mut buffer = [0u8; 128];
        let (ok, write_len) = primitive.dump(&mut buffer);
        assert!(ok, "Dump not ok");
        let (obj, read_len) = Primitive::load(&buffer);
        assert_eq!(write_len, read_len, "Amount written and amount read do not match");
        assert!(obj.is_some(), "Load not ok");
        if let Some(Primitive::Empty) = obj {
            //assert_eq!(data, result, "Data written and read does not match");
        } else {
            panic!("Read non-string primitive");
        }
    }
}
