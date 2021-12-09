use crate::software_vault::SoftwareVault;
use crate::VaultError;
use arrayref::array_ref;
use ockam_core::compat::vec::Vec;
use ockam_core::Result;
use ockam_vault_core::{
    Secret, SecretAttributes, SecretType, AES128_SECRET_LENGTH,
    AES256_SECRET_LENGTH,
};

use sha2::{Digest, Sha256};

impl SoftwareVault {
    pub(crate) fn sha256_sync(&self, data: &[u8]) -> Result<[u8; 32]> {
        let digest = Sha256::digest(data);
        Ok(*array_ref![digest, 0, 32])
    }

    /// Compute sha256.
    /// Salt and Ikm should be of Buffer type.
    /// Output secrets should be only of type Buffer or AES
    pub(crate) fn hkdf_sha256_sync(
        &self,
        salt: &Secret,
        info: &[u8],
        ikm: Option<&Secret>,
        output_attributes: Vec<SecretAttributes>,
    ) -> Result<Vec<Secret>> {
        let storage = self.inner.read();
        let ikm: Result<&[u8]> = match ikm {
            Some(ikm) => {
                let ikm = storage.get_entry(ikm)?;
                if ikm.key_attributes().stype() == SecretType::Buffer {
                    Ok(ikm.key().as_ref())
                } else {
                    Err(VaultError::InvalidKeyType.into())
                }
            }
            None => Ok(&[0u8; 0]),
        };

        let ikm = ikm?;

        let salt = storage.get_entry(salt)?;

        if salt.key_attributes().stype() != SecretType::Buffer {
            return Err(VaultError::InvalidKeyType.into());
        }

        // FIXME: Doesn't work for secrets with size more than 32 bytes
        let okm_len = output_attributes.len() * 32;

        let okm = {
            let mut okm = vec![0u8; okm_len];
            let prk = hkdf::Hkdf::<Sha256>::new(Some(salt.key().as_ref()), ikm);

            prk.expand(info, okm.as_mut_slice())
                .map_err(|_| Into::<ockam_core::Error>::into(VaultError::HkdfExpandError))?;
            okm
        };

        let mut secrets = Vec::<Secret>::new();
        let mut index = 0;

        for attributes in output_attributes {
            let length = attributes.length();
            if attributes.stype() == SecretType::Aes {
                if length != AES256_SECRET_LENGTH && length != AES128_SECRET_LENGTH {
                    return Err(VaultError::InvalidAesKeyLength.into());
                }
            } else if attributes.stype() != SecretType::Buffer {
                return Err(VaultError::InvalidHkdfOutputType.into());
            }
            let secret = &okm[index..index + length];
            let secret = self.secret_import_sync(secret, attributes)?;

            secrets.push(secret);
            index += 32;
        }

        Ok(secrets)
    }
}

