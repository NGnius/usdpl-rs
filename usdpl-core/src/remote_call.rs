use std::io::{Read, Write};

use crate::serdes::{DumpError, Dumpable, LoadError, Loadable, Primitive};

/// Remote call packet representing a function to call on the back-end, sent from the front-end
pub struct RemoteCall {
    /// The call id assigned by the front-end
    pub id: u64,
    /// The function's name
    pub function: String,
    /// The function's input parameters
    pub parameters: Vec<Primitive>,
}

impl Loadable for RemoteCall {
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let (id_num, len0) = u64::load(buffer)?;
        let (function_name, len1) = String::load(buffer)?;
        let (params, len2) = Vec::<Primitive>::load(buffer)?;
        Ok((
            Self {
                id: id_num,
                function: function_name,
                parameters: params,
            },
            len0 + len1 + len2,
        ))
    }
}

impl Dumpable for RemoteCall {
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
        let len0 = self.id.dump(buffer)?;
        let len1 = self.function.dump(buffer)?;
        let len2 = self.parameters.dump(buffer)?;
        Ok(len0 + len1 + len2)
    }
}

/// Remote call response packet representing the response from a remote call after the back-end has executed it.
pub struct RemoteCallResponse {
    /// The call id from the RemoteCall
    pub id: u64,
    /// The function's result
    pub response: Vec<Primitive>,
}

impl Loadable for RemoteCallResponse {
    fn load(buffer: &mut dyn Read) -> Result<(Self, usize), LoadError> {
        let (id_num, len0) = u64::load(buffer)?;
        let (response_var, len1) = Vec::<Primitive>::load(buffer)?;
        Ok((
            Self {
                id: id_num,
                response: response_var,
            },
            len0 + len1,
        ))
    }
}

impl Dumpable for RemoteCallResponse {
    fn dump(&self, buffer: &mut dyn Write) -> Result<usize, DumpError> {
        let len0 = self.id.dump(buffer)?;
        let len1 = self.response.dump(buffer)?;
        Ok(len0 + len1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_call_idempotence_test() {
        let call = RemoteCall{
            id: 42,
            function: "something very long just in case this causes unexpected issues".into(),
            parameters: vec!["param1".into(), 42f64.into()],
        };

        let mut buffer = String::with_capacity(crate::socket::PACKET_BUFFER_SIZE);
        let len = call.dump_base64(&mut buffer).unwrap();

        println!("base64 dumped: `{}` (len: {})", buffer, len);

        let (loaded_call, loaded_len) = RemoteCall::load_base64(buffer.as_bytes()).unwrap();
        assert_eq!(len, loaded_len, "Expected load and dump lengths to match");

        assert_eq!(loaded_call.id, call.id, "RemoteCall.id does not match");
        assert_eq!(loaded_call.function, call.function, "RemoteCall.function does not match");
        if let Primitive::String(loaded) = &loaded_call.parameters[0] {
            if let Primitive::String(original) = &call.parameters[0] {
                assert_eq!(loaded, original, "RemoteCall.parameters[0] does not match");
            } else {
                panic!("Original call parameter 0 is not String")
            }
        } else {
            panic!("Loaded call parameter 0 is not String")
        }
        if let Primitive::F64(loaded) = &loaded_call.parameters[1] {
            if let Primitive::F64(original) = &call.parameters[1] {
                assert_eq!(loaded, original, "RemoteCall.parameters[1] does not match");
            } else {
                panic!("Original call parameter 1 is not f64")
            }
        } else {
            panic!("Loaded call parameter 1 is not f64")
        }
    }
}
