use sha2::Sha256;
use hmac::Hmac;

type HmacSha256 = Hmac<Sha256>;

pub fn verify(code: &str, body:&str, key: &str) -> Result<bool,_> {

    // parse a[0] into a byte array
    let code = hex::decode(code);
    
    // initialize hmac code with secret key
    let mut hmac = HmacSha256::new_varkey(b"mysecretkey");
    
    // verify body
    // https://docs.rs/hmac/0.10.1/hmac/
    hmac.update(body);
    hmac.verify(&code)
}
