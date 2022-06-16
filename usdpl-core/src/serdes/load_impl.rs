use super::{LoadError, Loadable};

impl Loadable for String {
    fn load(buffer: &[u8]) -> Result<(Self, usize), LoadError> {
        if buffer.len() < 4 {
            return Err(LoadError::TooSmallBuffer);
        }
        let mut u32_bytes: [u8; 4] = [u8::MAX; 4];
        u32_bytes.copy_from_slice(&buffer[..4]);
        let str_size = u32::from_le_bytes(u32_bytes) as usize;
        Ok((
            Self::from_utf8_lossy(&buffer[4..str_size + 4]).into_owned(),
            str_size + 4,
        ))
    }
}

impl<T: Loadable> Loadable for Vec<T> {
    fn load(buffer: &[u8]) -> Result<(Self, usize), LoadError> {
        if buffer.len() < 4 {
            return Err(LoadError::TooSmallBuffer);
        }
        let mut u32_bytes: [u8; 4] = [u8::MAX; 4];
        u32_bytes.copy_from_slice(&buffer[..4]);
        let count = u32::from_le_bytes(u32_bytes) as usize;
        let mut cursor = 4;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            let (obj, len) = T::load(&buffer[cursor..])?;
            cursor += len;
            items.push(obj);
        }
        Ok((items, cursor))
    }
}

impl Loadable for bool {
    fn load(buffer: &[u8]) -> Result<(Self, usize), LoadError> {
        if buffer.len() < 1 {
            return Err(LoadError::TooSmallBuffer);
        }
        Ok((buffer[0] != 0, 1))
    }
}

impl Loadable for u8 {
    fn load(buffer: &[u8]) -> Result<(Self, usize), LoadError> {
        if buffer.len() < 1 {
            return Err(LoadError::TooSmallBuffer);
        }
        Ok((buffer[0], 1))
    }
}

impl Loadable for i8 {
    fn load(buffer: &[u8]) -> Result<(Self, usize), LoadError> {
        if buffer.len() < 1 {
            return Err(LoadError::TooSmallBuffer);
        }
        Ok((i8::from_le_bytes([buffer[0]]), 1))
    }
}

macro_rules! int_impl {
    ($type:ty, $size:literal) => {
        impl Loadable for $type {
            fn load(buffer: &[u8]) -> Result<(Self, usize), LoadError> {
                if buffer.len() < $size {
                    return Err(LoadError::TooSmallBuffer);
                }
                let mut bytes: [u8; $size] = [u8::MAX; $size];
                bytes.copy_from_slice(&buffer[..$size]);
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

    macro_rules! test_impl {
        ($fn_name:ident, $data:expr, $type:ty, $expected_len:literal, $expected_load:expr) => {
            #[test]
            fn $fn_name() {
                let buffer = $data;
                let (obj, read_len) = <$type>::load(&buffer).expect("Load not ok");
                assert_eq!(read_len, $expected_len, "Wrong amount read");
                assert_eq!(obj, $expected_load, "Loaded value not as expected");
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
