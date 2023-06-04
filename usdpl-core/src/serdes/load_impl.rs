use std::io::Read;

use super::{LoadError, Loadable};

impl Loadable for String {
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let mut u32_bytes: [u8; 4] = [u8::MAX; 4];
        buffer.read_exact(&mut u32_bytes).map_err(LoadError::Io)?;
        let str_size = u32::from_le_bytes(u32_bytes) as usize;
        //let mut str_buf = String::with_capacity(str_size);
        let mut str_buf = Vec::with_capacity(str_size);
        let mut byte_buf = [u8::MAX; 1];
        for _ in 0..str_size {
            buffer.read_exact(&mut byte_buf).map_err(LoadError::Io)?;
            str_buf.push(byte_buf[0]);
        }
        //let size2 = buffer.read_to_string(&mut str_buf).map_err(LoadError::Io)?;
        Ok((
            String::from_utf8(str_buf).map_err(|_| LoadError::InvalidData)?,
            str_size + 4,
        ))
    }
}

impl<T: Loadable> Loadable for Vec<T> {
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let mut u32_bytes: [u8; 4] = [u8::MAX; 4];
        buffer.read_exact(&mut u32_bytes).map_err(LoadError::Io)?;
        let count = u32::from_le_bytes(u32_bytes) as usize;
        let mut cursor = 4;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            let (obj, len) = T::load(buffer)?;
            cursor += len;
            items.push(obj);
        }
        Ok((items, cursor))
    }
}

impl<T0: Loadable, T1: Loadable> Loadable for (T0, T1) {
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let (t0, len0) = T0::load(buffer)?;
        let (t1, len1) = T1::load(buffer)?;
        Ok(((t0, t1), len0 + len1))
    }
}

impl<T0: Loadable, T1: Loadable, T2: Loadable> Loadable for (T0, T1, T2) {
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let (t0, len0) = T0::load(buffer)?;
        let (t1, len1) = T1::load(buffer)?;
        let (t2, len2) = T2::load(buffer)?;
        Ok(((t0, t1, t2), len0 + len1 + len2))
    }
}

impl<T0: Loadable, T1: Loadable, T2: Loadable, T3: Loadable> Loadable for (T0, T1, T2, T3) {
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let (t0, len0) = T0::load(buffer)?;
        let (t1, len1) = T1::load(buffer)?;
        let (t2, len2) = T2::load(buffer)?;
        let (t3, len3) = T3::load(buffer)?;
        Ok(((t0, t1, t2, t3), len0 + len1 + len2 + len3))
    }
}

impl<T0: Loadable, T1: Loadable, T2: Loadable, T3: Loadable, T4: Loadable> Loadable
    for (T0, T1, T2, T3, T4)
{
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let (t0, len0) = T0::load(buffer)?;
        let (t1, len1) = T1::load(buffer)?;
        let (t2, len2) = T2::load(buffer)?;
        let (t3, len3) = T3::load(buffer)?;
        let (t4, len4) = T4::load(buffer)?;
        Ok(((t0, t1, t2, t3, t4), len0 + len1 + len2 + len3 + len4))
    }
}

impl Loadable for bool {
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let mut byte = [u8::MAX; 1];
        buffer.read_exact(&mut byte).map_err(LoadError::Io)?;
        Ok((byte[0] != 0, 1))
    }
}

impl Loadable for u8 {
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let mut byte = [u8::MAX; 1];
        buffer.read_exact(&mut byte).map_err(LoadError::Io)?;
        Ok((byte[0], 1))
    }
}

impl Loadable for i8 {
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let mut byte = [u8::MAX; 1];
        buffer.read_exact(&mut byte).map_err(LoadError::Io)?;
        Ok((i8::from_le_bytes(byte), 1))
    }
}

macro_rules! int_impl {
    ($type:ty, $size:literal) => {
        impl Loadable for $type {
            fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
                let mut bytes: [u8; $size] = [u8::MAX; $size];
                buffer.read_exact(&mut bytes).map_err(LoadError::Io)?;
                let i = <$type>::from_le_bytes(bytes);
                Ok((i, $size))
            }
        }
    };
}

int_impl! {u16, 2}
int_impl! {u32, 4}
int_impl! {u64, 8}
int_impl! {u128, 16}

int_impl! {i16, 2}
int_impl! {i32, 4}
int_impl! {i64, 8}
int_impl! {i128, 16}

int_impl! {f32, 4}
int_impl! {f64, 8}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    macro_rules! test_impl {
        ($fn_name:ident, $data:expr, $type:ty, $expected_len:literal, $expected_load:expr) => {
            #[test]
            fn $fn_name() {
                let buffer_data = $data;
                let mut buffer = Vec::with_capacity(buffer_data.len());
                buffer.extend_from_slice(&buffer_data);
                let (obj, read_len) = <$type>::load(&mut Cursor::new(buffer)).expect("Load not ok");
                assert_eq!(read_len, $expected_len, "Wrong amount read");
                assert_eq!(obj, $expected_load, "Loaded value not as expected");
                println!("Loaded {:?}", obj);
            }
        };
    }

    test_impl! {string_load_test, [4u8, 0, 0, 0, 116, 101, 115, 116, 0, 128], String, 8, "test"}
    test_impl! {
        vec_load_test,
        [3u8, 0, 0, 0, 0, 0, 0, 0, 5, 0, 0, 0, 116, 101, 115, 116, 49, 5, 0, 0, 0, 116, 101, 115, 116, 50],
        Vec<String>,
        26,
        vec![
            "".to_string(),
            "test1".to_string(),
            "test2".to_string()
        ]
    }

    test_impl! {tuple2_load_test, [0, 1], (u8, u8), 2, (0, 1)}

    test_impl! {bool_true_load_test, [1], bool, 1, true}
    test_impl! {bool_false_load_test, [0], bool, 1, false}

    // testing macro-generated code isn't particularly useful, but do it anyway

    test_impl! {u8_load_test, [42], u8, 1, 42u8}
    test_impl! {u16_load_test, [42, 0], u16, 2, 42u16}
    test_impl! {u32_load_test, [42, 0, 0, 0], u32, 4, 42u32}
    test_impl! {u64_load_test, [42, 0, 0, 0, 0, 0, 0, 0], u64, 8, 42u64}
    test_impl! {u128_load_test, [42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], u128, 16, 42u128}

    test_impl! {i8_load_test, [42], i8, 1, 42i8}
    test_impl! {i16_load_test, [42, 0], i16, 2, 42i16}
    test_impl! {i32_load_test, [42, 0, 0, 0], i32, 4, 42i32}
    test_impl! {i64_load_test, [42, 0, 0, 0, 0, 0, 0, 0], i64, 8, 42i64}
    test_impl! {i128_load_test, [42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], i128, 16, 42i128}

    test_impl! {f32_load_test, [0, 0, 40, 66], f32, 4, 42f32}
    test_impl! {f64_load_test, [0, 0, 0, 0, 0, 0, 69, 64], f64, 8, 42f64}
}
