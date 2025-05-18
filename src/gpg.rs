use eyre::{Result, eyre};
use sequoia_openpgp::{
    Cert, KeyHandle, KeyID,
    cert::CertBuilder,
    crypto::SessionKey,
    packet::{
        Key, PKESK, SKESK,
        key::{SecretParts, UnspecifiedRole},
    },
    parse::{
        Parse,
        stream::{DecryptionHelper, DecryptorBuilder, MessageStructure, VerificationHelper},
    },
    policy::StandardPolicy,
    serialize::stream::{Armorer, Encryptor, LiteralWriter, Message},
    types::SymmetricAlgorithm,
};
use std::{
    collections::HashMap,
    io::Write,
    path::Path,
    sync::Arc,
};
use tracing::info;

/// Generates a new OpenPGP key with an encryption subkey,
/// protected by the given password.
pub fn generate_key_with_password(userid: String, password: &str) -> Result<Cert> {
    let (cert, _revocation_cert) = CertBuilder::new()
        .add_userid(userid)
        .set_password(Some(password.into()))
        .add_transport_encryption_subkey()
        .add_storage_encryption_subkey()
        .set_validity_period(None)
        .set_exportable(true)
        .generate()
        .map_err(|e| eyre!(e))?;

    Ok(cert)
}

/// Loads a PGP key (can be public or secret) from a file path,
/// validates it, and extracts a GpgIdentity (primary key fingerprint and user IDs).
/// This public key cert will be used as the recipient.
pub fn load_and_validate_key_from_file(key_file_path: &Path) -> Result<Cert> {
    let key_file_path = Path::new(key_file_path);
    if !key_file_path.exists() {
        return Err(eyre!("Key file not found at: {:?}", key_file_path));
    }

    let cert = Cert::from_file(key_file_path).map_err(|_| {
        eyre!(format!(
            "Failed to load PGP key from file: {:?}",
            key_file_path
        ))
    })?;

    let p = StandardPolicy::new();

    let primary_fingerprint = cert.fingerprint().to_hex().to_uppercase();

    let user_ids: Vec<String> = cert.userids().map(|uid| uid.userid().to_string()).collect();

    if user_ids.is_empty() {
        return Err(eyre!("Key '{}' has no user IDs.", primary_fingerprint));
    }

    // Check encryption capability
    let can_encrypt = cert
        .keys()
        .with_policy(&p, None)
        .any(|ka| ka.for_storage_encryption() || ka.for_transport_encryption());

    if !can_encrypt {
        return Err(eyre!(
            "The provided key (Fingerprint: {}) does not have an encryption-capable component.",
            primary_fingerprint
        ));
    }

    info!(
        "Successfully loaded key: {} (User IDs: {:?})",
        primary_fingerprint, user_ids
    );

    Ok(cert)
}

/// Encrypts data for the recipient identified by the Cert.
/// The `recipient_cert` is the public key we loaded earlier.
pub fn encrypt_data(data: &[u8], recipient: &Cert) -> Result<Vec<u8>> {
    let p = StandardPolicy::new();

    let recipients = recipient
        .keys()
        .with_policy(&p, None)
        .supported()
        .alive()
        .revoked(false)
        .for_storage_encryption();

    let mut sink = Vec::new();

    let message = Message::new(&mut sink);
    let message = Armorer::new(message)
        .build()
        .map_err(|e| eyre!("Failed to build Armorer: {}", e))?;
    let message = Encryptor::for_recipients(message, recipients)
        .build()
        .map_err(|e| eyre!("Failed to create Encryptor: {}", e))?;

    let mut message = LiteralWriter::new(message)
        .build()
        .map_err(|e| eyre!("Failed to create LiteralWriter: {}", e))?;
    message
        .write_all(data)
        .map_err(|e| eyre!("Failed to write data: {}", e))?;
    message
        .finalize()
        .map_err(|e| eyre!("Failed to finalize the message: {}", e))?;

    Ok(sink)
}

/// Decrypts the given armored ciphertext using the recipient's TSK.
/// Prompts for password if the TSK is encrypted.
pub fn decrypt_data(recipient: &Cert, ciphertext: &[u8]) -> Result<Vec<u8>> {
    let p = &StandardPolicy::new();
    let mut decrypted_plaintext = Vec::new();

    let helper = Helper::new(recipient, || {
        rpassword::prompt_password("Enter password for PGP key: ")
            .map_err(|e| eyre!("Failed to read password: {}", e))
    })?;

    let mut decryptor = DecryptorBuilder::from_bytes(ciphertext)
        .map_err(|e| eyre!(e))?
        .with_policy(p, None, helper)
        .map_err(|e| eyre!("Failed to build decryptor: {}", e))?;

    std::io::copy(&mut decryptor, &mut decrypted_plaintext)?;

    Ok(decrypted_plaintext)
}

struct Helper {
    secret_keys: HashMap<KeyID, (Cert, Key<SecretParts, UnspecifiedRole>)>,
    key_identities: HashMap<KeyID, Arc<Cert>>,
    password: String,
}

impl Helper {
    /// Creates a new helper.
    /// `password_cb` is a function that will be called to get the password if a key is encrypted.
    fn new(secret: &Cert, password_cb: impl Fn() -> Result<String>) -> Result<Self> {
        let p = StandardPolicy::new();

        let mut keys = HashMap::new();
        let mut identities = HashMap::new();

        let cert = Arc::new(secret.clone());
        for ka in secret
            .keys()
            .secret()
            .with_policy(&p, None)
            .for_storage_encryption()
            .for_transport_encryption()
        {
            let id = ka.key().keyid();
            let key = ka.key();
            keys.insert(id.clone(), (secret.clone(), key.clone()));
            identities.insert(id.clone(), cert.clone());
        }

        let password = password_cb()?;

        Ok(Self {
            secret_keys: keys,
            key_identities: identities,
            password,
        })
    }
}

impl DecryptionHelper for Helper {
    fn decrypt(
        &mut self,
        pkesks: &[PKESK],
        _skesks: &[SKESK],
        sym_algo_pref: Option<SymmetricAlgorithm>,
        decrypt_to: &mut dyn FnMut(Option<SymmetricAlgorithm>, &SessionKey) -> bool,
    ) -> sequoia_openpgp::anyhow::Result<Option<Cert>> {
        for pkesk in pkesks {
            let keyid = KeyID::from(pkesk.recipient());
            if let Some((cert, key)) = self.secret_keys.get_mut(&keyid) {
                if !key.clone().has_unencrypted_secret() {
                    let password = self.password.clone();
                    let mut keypair = key
                        .clone()
                        .decrypt_secret(&password.into())?
                        .into_keypair()?;

                    if pkesk
                        .decrypt(&mut keypair, sym_algo_pref)
                        .map(|(algo, session_key)| decrypt_to(algo, &session_key))
                        .unwrap_or(false)
                    {
                        return Ok(Some(cert.clone()));
                    }
                }
            }
        }
        Ok(None)
    }
}

impl VerificationHelper for Helper {
    fn get_certs(&mut self, _ids: &[KeyHandle]) -> sequoia_openpgp::anyhow::Result<Vec<Cert>> {
        Ok(Vec::new())
    }

    fn check(&mut self, _structure: MessageStructure) -> sequoia_openpgp::anyhow::Result<()> {
        Ok(())
    }
}
