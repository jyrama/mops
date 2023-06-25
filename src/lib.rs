use std::{format, println, sync::Arc, collections::HashMap};

use aes_gcm::{AeadInPlace, KeyInit};
use azure_identity::DefaultAzureCredentialBuilder;
use azure_security_keyvault::prelude::{DecryptParameters, EncryptionAlgorithm};
use serde::Deserialize;

use base64::Engine;
use typenum::U32;

pub type SopsAES = aes_gcm::AesGcm<aes::Aes256, typenum::U32>;
type UnsupportedVault = Vec<serde_json::Map<String, serde_json::Value>>;

#[derive(Debug, Deserialize)]
pub struct SopsFile {
    sops: Sops,
    #[serde(flatten)]
    content: serde_json::Map<String, serde_json::Value>,
}

impl SopsFile {
    async fn get_ciphers(&self) -> Vec<SopsAES> {
        let mut ciphers: Vec<SopsAES> = Vec::new();

        // Check for Azure Key Vault backed keys
        if let Some(ref vec_akvs) = self.sops.azure_kv {
            let creds = Arc::new(
                DefaultAzureCredentialBuilder::new()
                    .exclude_managed_identity_credential()
                    .build(),
            );

            for akv in vec_akvs {
                let enc_key = base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .decode(akv.enc.clone())
                    .expect("failed to debase64 encrypted master key");

                let key_client =
                    azure_security_keyvault::KeyClient::new(&akv.vault_url, creds.clone())
                        .expect("failed to create a Key Client");

                let params = DecryptParameters {
                    ciphertext: enc_key,
                    decrypt_parameters_encryption:
                        azure_security_keyvault::prelude::DecryptParametersEncryption::Rsa(
                            azure_security_keyvault::prelude::RsaDecryptParameters {
                                algorithm: EncryptionAlgorithm::RsaOaep256,
                            },
                        ),
                };

                let master_key = key_client
                    .decrypt(akv.name.clone(), params)
                    .await
                    .expect("master key decryption failed")
                    .result;

                let cipher =
                    SopsAES::new_from_slice(&master_key).expect("failed to construct a cipher");

                ciphers.push(cipher);
            }
        }

        ciphers
    }

    pub fn get_content(&self, key: &str) -> EncryptedContent {
        let enc_stuff = self
            .content
            .get(key)
            .expect("failed to find the key from the file")
            .as_str()
            .expect("the value wasn't string");

        let line_regex = regex::Regex::new(r"^ENC\[AES256_GCM,data:(?P<data>(.+)),iv:(?P<iv>(.+)),tag:(?P<tag>(.+)),type:(?P<type>(.+))\]").expect("could not compile regex");
        let captures = line_regex.captures(enc_stuff).expect("bad format");

        let data = base64::engine::general_purpose::STANDARD
            .decode(&captures["data"])
            .expect("failed to debase64 data");
        let iv = base64::engine::general_purpose::STANDARD
            .decode(&captures["iv"])
            .expect("failed to debase64 iv");
        let tag_raw = base64::engine::general_purpose::STANDARD
            .decode(&captures["tag"])
            .expect("failed to debase64 tag");

        let nonce = aes_gcm::Nonce::from_slice(&iv);
        let tag = aes_gcm::Tag::from_slice(&tag_raw);

        EncryptedContent {
            data,
            nonce: *nonce,
            tag: *tag,
            path: key.to_string(),
        }
    }
}

pub struct EncryptedContent {
    data: Vec<u8>,
    nonce: aes_gcm::Nonce<U32>,
    tag: aes_gcm::Tag,
    path: String,
}

impl EncryptedContent {
    pub fn decrypt(&self, ciphers: &Vec<SopsAES>) -> String {
        ciphers
            .into_iter()
            .find_map(|cipher| {
                let mut buffer = self.data.clone();
                match cipher.decrypt_in_place_detached(
                    &self.nonce,
                    format!("{}:", self.path).as_bytes(),
                    &mut buffer,
                    &self.tag,
                ) {
                    Ok(_) => Some(String::from_utf8(buffer).expect("not utf-8 enough")),
                    Err(_) => None,
                }
            })
            .expect("no keys found")
    }
}

type UnsupportedVault = Vec<serde_json::Map<String, serde_json::Value>>;

#[derive(Debug, Deserialize)]
pub struct Sops {
    kms: Option<UnsupportedVault>, // won't actually be missing but instead filled with a null
    gcp_kms: Option<UnsupportedVault>,
    azure_kv: Option<Vec<AzureKV>>,
    hc_vault: Option<UnsupportedVault>,
    age: Option<UnsupportedVault>,
    lastmodified: String, // a timestamp
    mac: String,
    pgp: Option<UnsupportedVault>,
    unencrypted_suffix: String,
    version: String,
}

#[derive(Debug, Deserialize)]
pub struct AzureKV {
    vault_url: String,
    name: String,
    version: String,
    created_at: String,
    enc: String,
}

