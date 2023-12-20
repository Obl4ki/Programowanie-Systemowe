use sha_crypt::{sha512_crypt_b64, Sha512Params, ROUNDS_DEFAULT};

#[derive(Clone)]
pub struct EtcShadowEntry {
    pub salt: String,
    pub hash: String,
}

impl EtcShadowEntry {
    pub fn repr(&self) -> String {
        format!("$6${}${}", self.salt, self.hash)
    }
}

pub fn get_entry(password: &str, salt: &str) -> EtcShadowEntry {
    // https://docs.rs/sha-crypt/latest/sha_crypt/

    let params =
        Sha512Params::new(ROUNDS_DEFAULT).expect("Rounds are in bounds so this will never fail.");
    let hashed_password = sha512_crypt_b64(password.as_bytes(), salt.as_bytes(), &params)
        .expect("Hashing must be working.");

    EtcShadowEntry {
        hash: hashed_password,
        salt: String::from(salt),
    }
}
