use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

use crate::error::OdbcSecretsLibError;

pub(crate) fn connect(address: String, token: String) -> Result<VaultClient, OdbcSecretsLibError> {
    // Create a client
    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address(address)
            .token(token)
            .build()?,
    )?;
    // client.status()
    Ok(client)
}
