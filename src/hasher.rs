use hmac_sha256;

pub fn verify(code: &str, body:&str, key: &str) -> Result<bool,String> {

    // parse a[0] into a byte array
    let code = match hex::decode(code) {
        Ok(x) => x,
        Err(e) => {
            return Err(format!("Failed to decode sig on verify command: {}", e));
        },
    };
    
    // initialize and obtain hmac code with secret key
    let hmac = hmac_sha256::HMAC::mac(body.as_bytes(), key.as_bytes());
    
    // verify body
    // compare hmac and code
    let mut c:u8 = 0;
    let mut len = 0;
    for (a,b) in hmac.iter().zip(code.iter()) {
        c |= a ^ b;
        len += 1;
    }
    if len != hmac.len() || len != code.len() {
        return Err("Wrong key length".to_string())
    }
    match c {
        0 => Ok(true),
        _ => Err("No Match".to_string())
    }
}

#[cfg(test)] 
mod tests {
    use super::*;
    #[test]
    fn hash_check() {
        assert_eq!(verify("da9297f14ca0def5a52fef03453087a1e96275baae3017520e40201082fc005f", "The quick fox", "abc"),
                    Ok(true), "test good");
        assert_eq!(verify("cb7bd4271b5c6153de7d5b6e013062f4989d7aebdb1837e12636caede0a09c72", "The quick fox", "abc"),
                    Err("No Match".to_string()), "wrong hash");
        assert_eq!(verify("d9297f14ca0def5a52fef03453087a1e96275baae3017520e40201082fc005f", "The quick fox", "abc"),
                    Err("Failed to decode sig on verify command: Odd number of digits".to_string()), "short hash");  
        assert_eq!(verify("9297f14ca0def5a52fef03453087a1e96275baae3017520e40201082fc005f", "The quick fox", "abc"),
                    Err("Wrong key length".to_string()), "very short hash");  
        assert_eq!(verify("da9297f14ca0def5a52fef03453087a1e96275baae3017520e40201082fc00", "The quick fox", "abc"),
                    Err("Wrong key length".to_string()), "truncated hash");
        assert_eq!(verify("", "The quick fox", "abc"),
                    Err("Wrong key length".to_string()), "no hash");
        }
}