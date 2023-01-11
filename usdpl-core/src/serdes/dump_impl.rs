use std::io::Write;

use super::{DumpError, Dumpable};

impl Dumpable for String {
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
        let str_bytes = self.as_bytes();
        let len_bytes = (str_bytes.len() as u32).to_le_bytes();
        let size1 = buffer.write(&len_bytes).map_err(DumpError::Io)?;
        let size2 = buffer.write(&str_bytes).map_err(DumpError::Io)?;
        Ok(size1 + size2)
    }
}

impl<T: Dumpable> Dumpable for Vec<T> {
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
        let len_bytes = (self.len() as u32).to_le_bytes();
        let mut total = buffer.write(&len_bytes).map_err(DumpError::Io)?;
        for obj in self.iter() {
            let len = obj.dump(buffer)?;
            total += len;
        }
        Ok(total)
    }
}

impl<T0: Dumpable, T1: Dumpable> Dumpable for (T0, T1) {
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
        Ok(
            self.0.dump(buffer)?
            + self.1.dump(buffer)?
        )
    }
}

impl<T0: Dumpable, T1: Dumpable, T2: Dumpable> Dumpable for (T0, T1, T2) {
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
        Ok(
            self.0.dump(buffer)?
            + self.1.dump(buffer)?
            + self.2.dump(buffer)?
        )
    }
}

impl<T0: Dumpable, T1: Dumpable, T2: Dumpable, T3: Dumpable> Dumpable for (T0, T1, T2, T3) {
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
        Ok(
            self.0.dump(buffer)?
            + self.1.dump(buffer)?
            + self.2.dump(buffer)?
            + self.3.dump(buffer)?
        )
    }
}

impl<T0: Dumpable, T1: Dumpable, T2: Dumpable, T3: Dumpable, T4: Dumpable> Dumpable for (T0, T1, T2, T3, T4) {
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
        Ok(
            self.0.dump(buffer)?
            + self.1.dump(buffer)?
            + self.2.dump(buffer)?
            + self.3.dump(buffer)?
            + self.4.dump(buffer)?
        )
    }
}

impl Dumpable for bool {
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
        buffer.write(&[*self as u8]).map_err(DumpError::Io)
    }
}

impl Dumpable for u8 {
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
        buffer.write(&[*self]).map_err(DumpError::Io)
    }
}

/*impl Dumpable for i8 {
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
        buffer.write(&self.to_le_bytes()).map_err(DumpError::Io)
    }
}*/

macro_rules! int_impl {
    ($type:ty) => {
        impl Dumpable for $type {
            fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
                buffer.write(&self.to_le_bytes()).map_err(DumpError::Io)
            }
        }
    };
}

int_impl! {u16}
int_impl! {u32}
int_impl! {u64}
int_impl! {u128}

int_impl! {i8}
int_impl! {i16}
int_impl! {i32}
int_impl! {i64}
int_impl! {i128}

int_impl! {f32}
int_impl! {f64}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_impl {
        ($fn_name:ident, $data:expr, $expected_len:literal, $expected_dump:expr) => {
            #[test]
            fn $fn_name() {
                let data = $data;
                let mut buffer = Vec::with_capacity(128);
                let write_len = data.dump(&mut buffer).expect("Dump not ok");
                assert_eq!(write_len, $expected_len, "Wrong amount written");
                assert_eq!(&buffer[..write_len], $expected_dump);
                println!("Dumped {:?}", buffer.as_slice());
            }
        };
    }

    test_impl! {string_dump_test, "test".to_string(), 8, &[4, 0, 0, 0, 116, 101, 115, 116]}

    test_impl! {
        vec_dump_test,
        vec![
            "".to_string(),
            "test1".to_string(),
            "test2".to_string()
        ],
        26,
        &[3, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 49, 5, 0, 0, 0, 116, 101, 115, 116, 50]
    }

    test_impl! {tuple2_dump_test, (0u8, 1u8), 2, &[0, 1]}
    test_impl! {tuple3_dump_test, (0u8, 1u8, 2u8), 3, &[0, 1, 2]}
    test_impl! {tuple4_dump_test, (0u8, 1u8, 2u8, 3u8), 4, &[0, 1, 2, 3]}
    test_impl! {tuple5_dump_test, (0u8, 1u8, 2u8, 3u8, 4u8), 5, &[0, 1, 2, 3, 4]}

    test_impl! {bool_true_dump_test, true, 1, &[1]}
    test_impl! {bool_false_dump_test, false, 1, &[0]}

    // testing macro-generated code isn't particularly useful, but do it anyway

    test_impl! {u8_dump_test, 42u8, 1, &[42]}
    test_impl! {u16_dump_test, 42u16, 2, &[42, 0]}
    test_impl! {u32_dump_test, 42u32, 4, &[42, 0, 0, 0]}
    test_impl! {u64_dump_test, 42u64, 8, &[42, 0, 0, 0, 0, 0, 0, 0]}
    test_impl! {u128_dump_test, 42u128, 16, &[42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]}

    test_impl! {i8_dump_test, 42i8, 1, &[42]}
    test_impl! {i16_dump_test, 42i16, 2, &[42, 0]}
    test_impl! {i32_dump_test, 42i32, 4, &[42, 0, 0, 0]}
    test_impl! {i64_dump_test, 42i64, 8, &[42, 0, 0, 0, 0, 0, 0, 0]}
    test_impl! {i128_dump_test, 42i128, 16, &[42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]}

    test_impl! {f32_dump_test, 42f32, 4, &[0, 0, 40, 66]}
    test_impl! {f64_dump_test, 42f64, 8, &[0, 0, 0, 0, 0, 0, 69, 64]}
}
