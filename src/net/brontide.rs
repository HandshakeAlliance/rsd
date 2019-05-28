use chacha20_poly1305_aead;
use hkdf::Hkdf;
use sha2::{Digest, Sha256};

use crate::Buffer;

const ROTATION_INTERVAL: u32 = 1000;
// pub struct Key([u8; 32]);
//
// TODO need to reimplement all Buffers as their own type most likely.

//TODO does this need to be public?
struct CipherState {
    //Double check on the size here TODO
    nonce: u32, //96 bit?
    secret_key: Buffer,
    salt: Buffer,
    iv: Buffer, //Capped at 12 bytes though, so maybe we need a new type.
}

//TODO again check on whether these things need to be public or not.
impl CipherState {
    pub fn update(&mut self) -> Buffer {
        //Refer to above TODO
        self.iv.write_u32(self.nonce);
        self.iv
    }

    //Todo maybe this a new function.
    pub fn init_key(&mut self, key: Buffer) {
        self.secret_key = key;
        self.nonce = 0;
        self.update();
    }

    //New with salt
    pub fn init_salt(&mut self, key: Buffer, salt: Buffer) {
        self.salt = salt;
        self.init_key(key);
    }

    pub fn rotate_key(&mut self) {
        let info = Buffer::new();
        let old = self.secret_key;
        let (salt, next) = expand(old, self.salt, info);

        self.salt = Buffer::from(salt);
        self.init_key(Buffer::from(next))
    }

    //TODO this needs heavy testing.
    pub fn encrypt(&mut self, pt: Buffer, ad: Buffer) -> Buffer {
        let mut ciphertext = Vec::with_capacity(pt.len());

        //TODO can't unwrap, need actual error handling here
        let tag =
            chacha20_poly1305_aead::encrypt(&self.secret_key, &self.iv, &ad, &pt, &mut ciphertext)
                .unwrap();

        self.nonce += 1;
        self.update();

        if self.nonce == ROTATION_INTERVAL {
            self.rotate_key();
        }

        Buffer::from(tag.to_vec())
    }

    pub fn decrypt(&mut self, ct: Buffer, tag: Buffer, ad: Buffer) -> bool {
        let mut plaintext = Vec::with_capacity(ct.len());

        let result = chacha20_poly1305_aead::decrypt(
            &self.secret_key,
            &self.iv,
            &ad,
            &tag,
            &ct,
            &mut plaintext,
        );

        match result {
            Err(_) => false,
            Ok(_) => {
                self.nonce += 1;
                self.update();

                if self.nonce == ROTATION_INTERVAL {
                    self.rotate_key();
                }

                true
            }
        }
    }
}

fn expand(secret: Buffer, salt: Buffer, info: Buffer) -> (Vec<u8>, Vec<u8>) {
    //hk.prk
    let hk = Hkdf::<Sha256>::extract(Some(&salt), &secret);
    let mut out = [0u8; 64];
    //TODO remove unwrap
    hk.expand(&info, &mut out).unwrap();

    //TODO double check this
    (out[0..32].to_vec(), out[32..64].to_vec())
}

pub struct SymmetricState {
    cipher: CipherState,
    chain: Buffer,  // chaining key
    temp: Buffer,   // temp key
    digest: Buffer, // handshake digest
}

impl SymmetricState {
    pub fn init_symmetric(&mut self, protocol_name: &str) {
        //I think this has to be a set size Buffer.
        let empty = Buffer::new();
        let proto = Buffer::from(protocol_name);

        let digest = Sha256::digest(&proto);
        self.digest = Buffer::from(digest.as_slice().to_vec());
        self.chain = self.digest;
        self.cipher.init_key(empty);
    }

    pub fn mix_key(&mut self, input: Buffer) {
        //I think this has to be a set size Buffer.
        let info = Buffer::new();
        let secret = input;
        let salt = self.chain;

        let (chain, temp) = expand(secret, salt, info);

        self.chain = Buffer::from(chain);
        self.temp = Buffer::from(temp);

        self.cipher.init_key(self.temp);
    }

    //TODO review
    pub fn mix_digest(&mut self, data: Buffer, tag: Buffer) -> Buffer {
        let mut hasher = Sha256::new();

        hasher.input(self.digest);
        hasher.input(data);
        hasher.input(tag);

        let result = hasher.result();

        Buffer::from(result.as_slice().to_vec())
    }

    pub fn mix_hash(&mut self, data: Buffer, tag: Buffer) {
        self.digest = self.mix_digest(data, tag);
    }

    //pt = plaintext, let's make that more verbose TODO so the code is more readable.
    pub fn encrypt_hash(&mut self, pt: Buffer) -> Buffer {
        let tag = self.cipher.encrypt(pt, self.digest);

        self.mix_hash(pt, tag);

        tag
    }

    //ct == CipherText, make this more verbose as above TODO
    pub fn decrypt_hash(&mut self, ct: Buffer, tag: Buffer) -> bool {
        let digest = self.mix_digest(ct, tag);

        let result = self.cipher.decrypt(ct, tag, self.digest);

        if result {
            self.digest = digest;
            true
        } else {
            false
        }
    }
}

//pub struct BrontideStream {}

//TODO maybe pull this entire package out into something else.
//impl BrontideStream {
//    //Function for connecting to outbound peers
//    //TODO new_outbound might actually be a better name here, but we'll see.
//    //One way to encapsulate this is to pass a mspc into here that listens to the stream of the
//    //socket.
//    pub fn connect() {

//    }
//}