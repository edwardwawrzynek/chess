use crate::error::Error;
use sha2::{Digest, Sha256};
use std::convert::TryFrom;
use uuid::Uuid;
use itertools::Itertools;

const HEX_CHARS: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

/// A hashed api key (safe to store in db + otherwise expose)
pub struct HashedApiKey([u8; 32]);

impl ToString for HashedApiKey {
    fn to_string(&self) -> String {
        // convert to hex string
        let mut res = String::new();
        for b in &self.0 {
            res.push(HEX_CHARS[(b & 0x0f) as usize]);
            res.push(HEX_CHARS[((b >> 4) & 0x0f) as usize]);
        }

        res
    }
}

impl TryFrom<String> for HashedApiKey {
    type Error = Error;

    fn try_from(key: String) -> Result<Self, Self::Error> {
        let mut res = [0u8; 32];

        if key.len() != 64 {
            Err(Error::MalformedApiKey)
        } else {
            for (i, (c1, c2)) in key.chars().tuples().enumerate() {
                let v1 = HEX_CHARS.iter().position(|c| *c == c1);
                let v2 = HEX_CHARS.iter().position(|c| *c == c2);
                match (v1, v2) {
                    (Some(i1), Some(i2)) => {
                        res[i] = ((i1 & 0xf) + ((i2 << 4) & 0xf0) & 0xff) as u8;
                    }
                    (_, _) => return Err(Error::MalformedApiKey),
                }
            }

            Ok(HashedApiKey(res))
        }
    }
}

/// A non hashed api key (not safe to expose in db)
pub struct ApiKey(Uuid);

impl ApiKey {
    pub fn hash(&self) -> HashedApiKey {
        let key_hash = Sha256::digest(self.0.as_bytes());

        let mut hash = [0; 32];
        for (i, b) in key_hash.as_slice().iter().enumerate() {
            hash[i] = *b;
        }

        HashedApiKey(hash)
    }

    pub fn new() -> ApiKey {
        ApiKey(Uuid::new_v4())
    }
}

impl ToString for ApiKey {
    fn to_string(&self) -> String {
        format!("{}", self.0.simple())
    }
}

impl From<&str> for ApiKey {
    fn from(str: &str) -> ApiKey {
        match Uuid::parse_str(str) {
            Ok(uuid) => ApiKey(uuid),
            Err(_) => ApiKey(Uuid::nil()),
        }
    }
}