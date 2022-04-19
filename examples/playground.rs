use std::mem;

fn main() -> Result<(), kv_server::error::Error> {
    // decode Signature -> &[u8]：
    let signature_base64: &str = "CNn87foZQt8AY+yRA/ys2/99zlD6gEnph3ujaIdQXxlKdHB41Ev+/rS/fzIULuWrljGreVbR/hRHL7RB51jIfRs=";  
    let decode_signature = base64::decode(signature_base64)?;
    let signature_u8_slice: &[u8] = decode_signature.as_ref(); 
    println!("signature_u8_slice size: {}", mem::size_of_val(signature_u8_slice));
    Ok(())
}