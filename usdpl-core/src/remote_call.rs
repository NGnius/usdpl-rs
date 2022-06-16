use crate::serdes::{DumpError, Dumpable, LoadError, Loadable, Primitive};

/// Remote call packet representing a function to call on the back-end, sent from the front-end
pub struct RemoteCall {
    pub id: u64,
    pub function: String,
    pub parameters: Vec<Primitive>,
}

impl Loadable for RemoteCall {
    fn load(buffer: &[u8]) -> Result<(Self, usize), LoadError> {
        let (id_num, len0) = u64::load(buffer)?;
        let (function_name, len1) = String::load(&buffer[len0..])?;
        let (params, len2) = Vec::<Primitive>::load(&buffer[len0 + len1..])?;
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
    fn dump(&self, buffer: &mut [u8]) -> Result<usize, DumpError> {
        let len0 = self.id.dump(buffer)?;
        let len1 = self.function.dump(&mut buffer[len0..])?;
        let len2 = self.parameters.dump(&mut buffer[len0 + len1..])?;
        Ok(len0 + len1 + len2)
    }
}

/// Remote call response packet representing the response from a remote call after the back-end has executed it.
pub struct RemoteCallResponse {
    pub id: u64,
    pub response: Vec<Primitive>,
}

impl Loadable for RemoteCallResponse {
    fn load(buffer: &[u8]) -> Result<(Self, usize), LoadError> {
        let (id_num, len0) = u64::load(buffer)?;
        let (response_var, len1) = Vec::<Primitive>::load(&buffer[len0..])?;
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
    fn dump(&self, buffer: &mut [u8]) -> Result<usize, DumpError> {
        let len0 = self.id.dump(buffer)?;
        let len1 = self.response.dump(&mut buffer[len0..])?;
        Ok(len0 + len1)
    }
}
