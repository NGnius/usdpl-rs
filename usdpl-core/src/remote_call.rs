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
