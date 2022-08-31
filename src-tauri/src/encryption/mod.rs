use aes_gcm::aead::{Aead, NewAead};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};

pub mod error;
use self::error::WrongPasswordError;

pub fn encrypt(plaintext: Vec<u8>, password: String) -> Vec<u8> {
    let salt = SaltString::new("saltsaltsaltsaltsaltsalt").unwrap();
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .hash
        .unwrap();

    let key = Key::from_slice(password_hash.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique nonce");
    cipher.encrypt(nonce, plaintext.as_ref()).unwrap()
}

pub fn decrypt(ciphertext: Vec<u8>, password: String) -> Result<Vec<u8>, WrongPasswordError> {
    let salt = SaltString::new("saltsaltsaltsaltsaltsalt").unwrap();
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .hash
        .unwrap();
    let key = Key::from_slice(password_hash.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"unique nonce");
    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref());
    match plaintext {
        Ok(plaintext) => Ok(plaintext),
        Err(_) => Err(WrongPasswordError {}),
    }
}

#[cfg(test)]
mod tests {
    use crate::encryption::{decrypt, encrypt};

    #[test]
    fn test_encrypt_decrypt_success() {
        let password = "password1234".to_string();
        let plaintext = "secret message";
        let ciphertext = encrypt(plaintext.into(), password.clone());
        let plaintext_dec = decrypt(ciphertext, password).unwrap();
        let plaintext_dec = String::from_utf8(plaintext_dec).unwrap();
        assert_eq!(plaintext, plaintext_dec);
    }
}
