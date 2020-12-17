use hmac_sha256;

pub fn verify(code: &str, body:&str, key: &str) -> Result<bool,_> {

    // parse a[0] into a byte array
    let code = match hex::decode(code) {
        Ok(x) => x,
        Err(e) => {
            return Err(format!("Failed to decode sig on verify command: {}", e));
        },
    };
    
    // TODO convert vec to byte[]
    // https://stackoverflow.com/questions/29570607/is-there-a-good-way-to-convert-a-vect-to-an-array
    
    // initialize and obtain hmac code with secret key
    let mut hmac = hmac_sha256::HMAC::mac(code, key.as_bytes());
    
    // verify body
    // https://docs.rs/hmac/0.10.1/hmac/
    /* hmac.update(body);
    hmac.verify(&code) */
    Ok(true)
}
