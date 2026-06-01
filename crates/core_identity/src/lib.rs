use ldap3::{LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
use std::env;
use tracing::warn;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationalUnit {
    pub name: String,
    pub dn: String,
}

pub struct LdapClient {
    url: String,
    bind_dn: String,
    password: String,
    base_dn: String,
}

impl LdapClient {
    pub fn new() -> Result<Self, anyhow::Error> {
        let url = env::var("LDAP_URL").unwrap_or_else(|_| "ldap://127.0.0.1:389".to_string());
        let bind_dn = env::var("LDAP_BIND_DN").unwrap_or_else(|_| "".to_string());
        let password = env::var("LDAP_PASSWORD").unwrap_or_else(|_| "".to_string());
        let base_dn = env::var("LDAP_BASE_DN").unwrap_or_else(|_| "DC=zeta,DC=local".to_string());

        Ok(Self {
            url,
            bind_dn,
            password,
            base_dn,
        })
    }

    pub async fn fetch_ous(&self) -> Result<Vec<OrganizationalUnit>, anyhow::Error> {
        if self.bind_dn.is_empty() {
            warn!("LDAP_BIND_DN is empty, AD integration might not be configured properly.");
            return Ok(vec![]);
        }

        let settings = LdapConnSettings::new().set_conn_timeout(std::time::Duration::from_secs(3));
        let (conn, mut ldap) = LdapConnAsync::with_settings(settings, &self.url).await?;
        
        // Spawn background connection handler
        ldap3::drive!(conn);

        // Bind
        ldap.simple_bind(&self.bind_dn, &self.password).await?.success()?;

        // Search for OUs
        // ObjectClass=organizationalUnit
        let (rs, _res) = ldap.search(
            &self.base_dn,
            Scope::Subtree,
            "(objectClass=organizationalUnit)",
            vec!["ou"]
        ).await?.success()?;

        let mut ous = Vec::new();
        for entry in rs {
            let search_entry = SearchEntry::construct(entry);
            let name = search_entry.attrs.get("ou").and_then(|v| v.first()).cloned().unwrap_or_else(|| "Unknown".to_string());
            ous.push(OrganizationalUnit {
                name,
                dn: search_entry.dn,
            });
        }

        let _ = ldap.unbind().await;
        Ok(ous)
    }

    /// Check if a given username belongs to the specified OU (by checking its DN)
    pub async fn check_user_in_ou(&self, username: &str, target_ou_dn: &str) -> Result<bool, anyhow::Error> {
        if self.bind_dn.is_empty() {
            return Ok(true); // If LDAP not configured, bypass
        }

        let settings = LdapConnSettings::new().set_conn_timeout(std::time::Duration::from_secs(3));
        let (conn, mut ldap) = LdapConnAsync::with_settings(settings, &self.url).await?;
        ldap3::drive!(conn);

        ldap.simple_bind(&self.bind_dn, &self.password).await?.success()?;

        // Search for user
        let filter = format!("(&(objectClass=user)(sAMAccountName={}))", username);
        let (rs, _res) = ldap.search(
            &self.base_dn,
            Scope::Subtree,
            &filter,
            vec!["distinguishedName"]
        ).await?.success()?;

        let mut is_authorized = false;
        if let Some(entry) = rs.into_iter().next() {
            let search_entry = SearchEntry::construct(entry);
            let user_dn = search_entry.dn.to_lowercase();
            let expected_ou = target_ou_dn.to_lowercase();
            // In AD, if a user is in an OU, their DN ends with the OU's DN
            if user_dn.ends_with(&expected_ou) {
                is_authorized = true;
            }
        }

        let _ = ldap.unbind().await;
        Ok(is_authorized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
