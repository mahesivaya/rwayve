#[cfg(test)]
mod tests {
    use super::*;

    fn set_test_key() {
        unsafe {
            std::env::set_var("AES_KEY", "0123456789abcdef0123456789abcdef");
        }
    }

    #[test]
    fn round_trip() {
        set_test_key();
        let plaintext = "the quick brown fox";
        let (nonce, cipher) = encrypt(plaintext).unwrap();
        assert_ne!(cipher, plaintext, "ciphertext must not equal plaintext");
        let decrypted = decrypt(&nonce, &cipher).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn each_call_uses_fresh_nonce() {
        set_test_key();
        let (n1, c1) = encrypt("same input").unwrap();
        let (n2, c2) = encrypt("same input").unwrap();
        assert_ne!(n1, n2, "nonces must differ");
        assert_ne!(c1, c2, "ciphertexts must differ for the same plaintext");
    }

    #[test]
    fn rejects_short_nonce() {
        set_test_key();
        let err = decrypt("AAAA", "any").unwrap_err();
        assert!(err.contains("Invalid nonce length"), "got: {err}");
    }

    #[test]
    fn rejects_empty_ciphertext() {
        set_test_key();
        let (nonce, _) = encrypt("hello").unwrap();
        let err = decrypt(&nonce, "").unwrap_err();
        assert!(err.contains("Empty"), "got: {err}");
    }

    #[test]
    fn rejects_tampered_ciphertext() {
        set_test_key();
        let (nonce, cipher) = encrypt("sensitive").unwrap();
        // Flip the first character of ciphertext.
        let mut bytes = cipher.into_bytes();
        bytes[0] = bytes[0].wrapping_add(1);
        let tampered = String::from_utf8(bytes).unwrap();
        assert!(decrypt(&nonce, &tampered).is_err());
    }
}
