use std::collections::HashMap;

use mops::SopsFile;

#[tokio::main]
async fn main() {
    let dada: SopsFile =
        serde_json::from_str(DADA_AKV).expect("deserializing a SOPS encoded file failed");

    let ciphers = dada.get_ciphers().await;

    let bsd: HashMap<String, String> = dada
        .content
        .keys()
        .into_iter()
        .map(|key| (key.clone(), dada.get_content(key).decrypt(&ciphers)))
        .collect();

    println!("{:#?}", bsd);
}

const DADA_AKV: &'static str = r#"
{
	"OAUTH_CLIENT_SECRET": "ENC[AES256_GCM,data:ClXNvw==,iv:6LmoXXpKWQ4VWG6KAqYiSGnCUYUvd3TW/Nc2RgHrFzI=,tag:hw91QeRwrrtfqjsQ844Uiw==,type:str]",
	"GITLAB_API_ACCESS_TOKEN": "ENC[AES256_GCM,data:9yfYh2aglw==,iv:jCwy3dAEF18B7TNBfy4Glb9hHQ2xPBueW+7UDPdK5uY=,tag:l7Yb6Jtjr103fly4bXOjDw==,type:str]",
	"DADA": "ENC[AES256_GCM,data:Otrr8Q==,iv:rOlsJxDIfAtLBXgh0wPzfDcZXjMpbM7CqUnFWc8SqZk=,tag:/5JbI2KV2/dkk0W++MgN6g==,type:str]",
	"sops": {
		"kms": null,
		"gcp_kms": null,
		"azure_kv": [
			{
				"vault_url": "https://mops-ci.vault.azure.net",
				"name": "sops-key",
				"version": "6cac3e56d9844703bdd908eb6d142b4a",
				"created_at": "2023-06-25T18:48:01Z",
				"enc": "dstUB1GhcOH_oQE_sBmO-KXQeaIrnPtVSRM9W2oYrGXnd1_Mp2vyagXKueUz8Cljv7N492nIWTQMwZyTaNorSCWKc1VBWD-vAep2tpOM3CtSbgKLobHTg8DHhrvyjY4DQromkAJHN4V7IDnvDiFGFJ-G7EeaKw_RCcxskuwJEzDylZrajtu7ZdyDROl5MxJoNIqOvLi-eFzYki2atuepnYyx7sEbAN6s71eSpjr6jGoZU4mWIQ9E3HBtMwJpMR4qPVxACiVjKsOSnsaJYS3at4ile0GYaBQhengLTRe5y2gm_2L0s7SsBXXnfX8pJk88Z_Kxb7KKP6CLjbEYk5rqCA"
			}
		],
		"hc_vault": null,
		"age": null,
		"lastmodified": "2023-06-25T18:48:03Z",
		"mac": "ENC[AES256_GCM,data:sf4dhi2Sl7wQEH3qa4BorGaENAuTTyQX3JamjcSS/ZVdZODCXyxszjSbgQ6rgEE0AIyC7OzemR+TbZrrYKOIFOoOiIAnYEovA/NTrWqKWpwv9SnCNcbyD0fFlzroKB1Gu0R2Gjg/vDJIplyX3r5ldfrsNVBhPXXNi2M61041XnQ=,iv:vC3iw1rglWAUks9xEJMoWk5PP1NdxIuVk3wBLW36bC4=,tag:RDhI2lmGnKLUt/HBqV5e8A==,type:str]",
		"pgp": null,
		"unencrypted_suffix": "_unencrypted",
		"version": "3.7.3"
	}
}"#;
