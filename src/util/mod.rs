use crate::error::Error;

pub fn base64_to_vec(b64: &String) -> Result<Vec<u8>, Error> {
    base64::decode(b64).map_err(|e| e.into())
}

pub fn vec_to_base64(bytes_vec: &Vec<u8>) -> String {
    base64::encode(bytes_vec)
}
