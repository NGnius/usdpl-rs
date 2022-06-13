use crate::serdes::{Primitive, Loadable, Dumpable};

/// Remote call packet representing a function to call on the back-end, sent from the front-end
pub struct RemoteCall {
    pub id: u64,
    pub function: String,
    pub parameters: Vec<Primitive>,
}

impl Loadable for RemoteCall {
    fn load(buffer: &[u8]) -> (Option<Self>, usize) {
        let (id_num, len0) = u64::load(buffer);
        if id_num.is_none() {
            return (None, len0);
        }
        let (function_name, len1) = String::load(&buffer[len0..]);
        if function_name.is_none() {
            return (None, len1);
        }
        let (params, len2) = Vec::<Primitive>::load(&buffer[len0+len1..]);
        if params.is_none() {
            return (None, len1 + len2);
        }
        (
            Some(Self {
                id: id_num.unwrap(),
                function: function_name.unwrap(),
                parameters: params.unwrap(),
            }),
            len0 + len1 + len2
        )
    }
}

impl Dumpable for RemoteCall {
    fn dump(&self, buffer: &mut [u8]) -> (bool, usize) {
        let (ok0, len0) = self.id.dump(buffer);
        if !ok0 {
            return (ok0, len0);
        }
        let (ok1, len1) = self.function.dump(&mut buffer[len0..]);
        if !ok1 {
            return (ok1, len1);
        }
        let (ok2, len2) = self.parameters.dump(&mut buffer[len0+len1..]);
        (ok2, len0 + len1 + len2)
    }
}

/// Remote call response packet representing the response from a remote call after the back-end has executed it.
pub struct RemoteCallResponse {
    pub id: u64,
    pub response: Vec<Primitive>,
}

impl Loadable for RemoteCallResponse {
    fn load(buffer: &[u8]) -> (Option<Self>, usize) {
        let (id_num, len0) = u64::load(buffer);
        if id_num.is_none() {
            return (None, len0);
        }
        let (response_var, len1) = Vec::<Primitive>::load(&buffer[len0..]);
        if response_var.is_none() {
            return (None, len1);
        }
        (
            Some(Self {
                id: id_num.unwrap(),
                response: response_var.unwrap(),
            }),
            len0 + len1
        )
    }
}

impl Dumpable for RemoteCallResponse {
    fn dump(&self, buffer: &mut [u8]) -> (bool, usize) {
        let (ok0, len0) = self.id.dump(buffer);
        if !ok0 {
            return (ok0, len0);
        }
        let (ok1, len1) = self.response.dump(&mut buffer[len0..]);
        (ok1, len0 + len1)
    }
}

