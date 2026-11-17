/*


████████╗██╗░░░██╗██████╗░░█████╗░██╗░██████╗████████╗  ██████╗░░░░░█████╗░░░░░█████╗░
╚══██╔══╝╚██╗░██╔╝██╔══██╗██╔══██╗██║██╔════╝╚══██╔══╝  ╚════██╗░░░██╔══██╗░░░██╔══██╗
░░░██║░░░░╚████╔╝░██████╔╝██║░░██║██║╚█████╗░░░░██║░░░  ░░███╔═╝░░░██║░░██║░░░██║░░██║
░░░██║░░░░░╚██╔╝░░██╔═══╝░██║░░██║██║░╚═══██╗░░░██║░░░  ██╔══╝░░░░░██║░░██║░░░██║░░██║
░░░██║░░░░░░██║░░░██║░░░░░╚█████╔╝██║██████╔╝░░░██║░░░  ███████╗██╗╚█████╔╝██╗╚█████╔╝
░░░╚═╝░░░░░░╚═╝░░░╚═╝░░░░░░╚════╝░╚═╝╚═════╝░░░░╚═╝░░░  ╚══════╝╚═╝░╚════╝░╚═╝░╚════╝░

Made with ♥ by tfaullk

*/

use std::fs;
use std::path::PathBuf;

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey, Verifier};
use rand::rngs::OsRng;

use crate::error::Result;

pub struct Identity {
    signing: SigningKey,
    pub verifying: VerifyingKey,
}

impl Identity {
    pub fn load_or_create() -> Result<Self> {
        let path = Self::path();
        if let Ok(bytes) = fs::read(&path) {
            if bytes.len() == 32 {
                let arr: [u8; 32] = bytes.as_slice().try_into().unwrap();
                let signing = SigningKey::from_bytes(&arr);
                let verifying = signing.verifying_key();
                return Ok(Self { signing, verifying });
            }
            return Err(crate::error::TypoistError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("identity key file {} is corrupt (expected 32 bytes)", path.display()),
            )));
        }
        let signing = SigningKey::generate(&mut OsRng);
        crate::config::paths::ensure_config_dir()?;
        write_private(&path, &signing.to_bytes())?;
        let verifying = signing.verifying_key();
        Ok(Self { signing, verifying })
    }

    fn path() -> PathBuf {
        crate::config::paths::config_dir().join("identity.key")
    }

    pub fn public_hex(&self) -> String {
        hex::encode(self.verifying.as_bytes())
    }

    pub fn sign(&self, msg: &[u8]) -> String {
        hex::encode(self.signing.sign(msg).to_bytes())
    }

    pub fn verify(verifying: &VerifyingKey, msg: &[u8], sig_hex: &str) -> bool {
        let Ok(bytes) = hex::decode(sig_hex) else { return false };
        let Ok(arr) = <[u8; 64]>::try_from(bytes.as_slice()) else { return false };
        let sig = Signature::from_bytes(&arr);
        verifying.verify(msg, &sig).is_ok()
    }
}

fn write_private(path: &PathBuf, data: &[u8]) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let mut f = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(path)?;
        f.write_all(data)?;
        let mut p = f.metadata()?.permissions();
        p.set_mode(0o600);
        fs::set_permissions(path, p)?;
        Ok(())
    }
    #[cfg(windows)]
    {
        fs::write(path, data)
    }
}
