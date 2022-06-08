use super::Dumpable;

impl Dumpable for String {
    fn dump(&self, buffer: &mut [u8]) -> (bool, usize) {
        let str_bytes = self.as_bytes();
        let len_bytes = (str_bytes.len() as u32).to_le_bytes();
        let total_len = str_bytes.len() + 4;
        if buffer.len() < total_len {
            return (false, 0);
        }
        (&mut buffer[..4]).copy_from_slice(&len_bytes);
        (&mut buffer[4..total_len]).copy_from_slice(str_bytes);
        (true, total_len)
    }
}

impl<T: Dumpable> Dumpable for Vec<T> {
    fn dump(&self, buffer: &mut [u8]) -> (bool, usize) {
        let len_bytes = (self.len() as u32).to_le_bytes();
        (&mut buffer[..4]).copy_from_slice(&len_bytes);
        let mut cursor = 4;
        for obj in self.iter() {
            let (ok, len) = obj.dump(&mut buffer[cursor..]);
            cursor += len;
            if !ok {
                return (false, cursor);
            }
        }
        (true, cursor)
    }
}

impl Dumpable for bool {
    fn dump(&self, buffer: &mut [u8]) -> (bool, usize) {
        if buffer.len() < 1 {
            return (false, 0);
        }
        buffer[0] = *self as u8;
        (true, 1)
    }
}

impl Dumpable for u8 {
    fn dump(&self, buffer: &mut [u8]) -> (bool, usize) {
        if buffer.len() < 1 {
            return (false, 0);
        }
        buffer[0] = *self;
        (true, 1)
    }
}

impl Dumpable for i8 {
    fn dump(&self, buffer: &mut [u8]) -> (bool, usize) {
        if buffer.len() < 1 {
            return (false, 0);
        }
        buffer[0] = self.to_le_bytes()[0];
        (true, 1)
    }
}

macro_rules! int_impl {
    ($type:ty, $size:literal) => {
        impl Dumpable for $type {
            fn dump(&self, buffer: &mut [u8]) -> (bool, usize) {
                if buffer.len() < $size {
                    return (false, 0);
                }
                (&mut buffer[..$size]).copy_from_slice(&self.to_le_bytes());
                (true, $size)
            }
        }
    }
}

int_impl!{u16, 2}
int_impl!{u32, 4}
int_impl!{u64, 8}
int_impl!{u128, 16}

int_impl!{i16, 2}
int_impl!{i32, 4}
int_impl!{i64, 8}
int_impl!{i128, 16}

int_impl!{f32, 4}
int_impl!{f64, 8}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_impl {
        ($fn_name:ident, $data:expr, $expected_len:literal, $expected_dump:expr) => {
            #[test]
            fn $fn_name() {
                let data = $data;
                let mut buffer = [0u8; 128];
                let (ok, write_len) = data.dump(&mut buffer);
                assert!(ok, "Dump not ok");
                assert_eq!(write_len, $expected_len, "Wrong amount written");
                assert_eq!(&buffer[..write_len], $expected_dump);
            }
        }
    }

    test_impl!{string_dump_test, "test".to_string(), 8, &[4, 0, 0, 0, 116, 101, 115, 116]}

    test_impl!{
        vec_dump_test,
        vec![
            "".to_string(),
            "test1".to_string(),
            "test2".to_string()
        ],
        26,
        &[3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 49, 5, 0, 0, 0, 116, 101, 115, 116, 50]
    }

    test_impl!{bool_true_dump_test, true, 1, &[1]}
    test_impl!{bool_false_dump_test, false, 1, &[0]}

    // testing macro-generated code isn't particularly useful, but do it anyway

    test_impl!{u8_dump_test, 42u8, 1, &[42]}
    test_impl!{u16_dump_test, 42u16, 2, &[42, 0]}
    test_impl!{u32_dump_test, 42u32, 4, &[42, 0, 0, 0]}
    test_impl!{u64_dump_test, 42u64, 8, &[42, 0, 0, 0, 0, 0, 0, 0]}
    test_impl!{u128_dump_test, 42u128, 16, &[42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]}

    test_impl!{i8_dump_test, 42i8, 1, &[42]}
    test_impl!{i16_dump_test, 42i16, 2, &[42, 0]}
    test_impl!{i32_dump_test, 42i32, 4, &[42, 0, 0, 0]}
    test_impl!{i64_dump_test, 42i64, 8, &[42, 0, 0, 0, 0, 0, 0, 0]}
    test_impl!{i128_dump_test, 42i128, 16, &[42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]}

    test_impl!{f32_dump_test, 42f32, 4, &[0, 0, 40, 66]}
    test_impl!{f64_dump_test, 42f64, 8, &[0, 0, 0, 0, 0, 0, 69, 64]}
}
