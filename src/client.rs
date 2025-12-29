use std::collections::HashMap;
use std::time::Duration;

use reqwest::{
    Client, RequestBuilder,
    header::{HeaderName, HeaderValue},
};
use url::Url;

use crate::{
    AuthorizationRequest, AuthorizationResponse, OAuthError, OAuthProvider, PkcePair,
    TokenRequestFormat, TokenResponse,
};
#[cfg(feature = "local-server")]
use crate::{LocalServer, LocalServerConfig};

#[derive(Debug, Clone)]
pub struct OAuthClientConfig {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub authorize_params: Vec<(String, String)>,
    pub token_params: Vec<(String, String)>,
    pub timeout: Option<Duration>,
    #[cfg(feature = "local-server")]
    pub local_server: Option<LocalServerConfig>,
}

impl OAuthClientConfig {
    pub fn new(client_id: impl Into<String>, redirect_uri: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: None,
            redirect_uri: redirect_uri.into(),
            scope: None,
            authorize_params: Vec::new(),
            token_params: Vec::new(),
            timeout: None,
            #[cfg(feature = "local-server")]
            local_server: None,
        }
    }

    pub fn with_client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_secret = Some(client_secret.into());
        self
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    #[cfg(feature = "local-server")]
    pub fn with_local_server_config(mut self, local_server: LocalServerConfig) -> Self {
        self.redirect_uri = local_server.redirect_uri();
        self.local_server = Some(local_server);
        self
    }

    pub fn with_authorize_param(
        mut self,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.authorize_params.push((key.into(), value.into()));
        self
    }

    pub fn with_token_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.token_params.push((key.into(), value.into()));
        self
    }
}

#[derive(Debug, Clone)]
pub struct OAuthClient<P: OAuthProvider> {
    provider: P,
    config: OAuthClientConfig,
    http: Client,
}

impl<P: OAuthProvider> OAuthClient<P> {
    pub fn new(provider: P, config: OAuthClientConfig) -> Result<Self, OAuthError> {
        let mut builder = Client::builder();
        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }
        let http = builder.build()?;
        Ok(Self {
            provider,
            config,
            http,
        })
    }

    pub fn with_http_client(provider: P, config: OAuthClientConfig, http: Client) -> Self {
        Self {
            provider,
            config,
            http,
        }
    }

    pub fn provider(&self) -> &P {
        &self.provider
    }

    pub fn config(&self) -> &OAuthClientConfig {
        &self.config
    }

    pub fn authorization_url(&self) -> Result<AuthorizationRequest, OAuthError> {
        self.authorization_url_with_state(None)
    }

    pub fn authorization_url_with_state(
        &self,
        state: Option<String>,
    ) -> Result<AuthorizationRequest, OAuthError> {
        let pkce = PkcePair::generate()?;
        let state = state.unwrap_or_else(|| pkce.code_verifier.clone());
        let scope = self
            .config
            .scope
            .as_deref()
            .unwrap_or(self.provider.default_scope());

        let mut params: HashMap<String, String> = HashMap::new();
        for (key, value) in self.provider.authorize_params() {
            params.insert(key, value);
        }
        for (key, value) in &self.config.authorize_params {
            params.insert(key.clone(), value.clone());
        }

        params.insert("response_type".to_string(), "code".to_string());
        params.insert("client_id".to_string(), self.config.client_id.clone());
        params.insert("redirect_uri".to_string(), self.config.redirect_uri.clone());
        params.insert("scope".to_string(), scope.to_string());
        params.insert("code_challenge".to_string(), pkce.code_challenge.clone());
        params.insert("code_challenge_method".to_string(), "S256".to_string());
        params.insert("state".to_string(), state.clone());

        let mut url = Url::parse(self.provider.authorize_url())?;
        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in params {
                pairs.append_pair(&key, &value);
            }
        }

        Ok(AuthorizationRequest {
            authorization_url: url.to_string(),
            pkce,
            state,
            scope: scope.to_string(),
        })
    }

    #[cfg(feature = "local-server")]
    pub async fn run_local_flow<F>(&self, on_authorize: F) -> Result<TokenResponse, OAuthError>
    where
        F: FnOnce(&AuthorizationRequest) -> Result<(), OAuthError>,
    {
        let auth = self.authorization_url()?;
        let expected_state = auth.state.clone();
        let code_verifier = auth.pkce.code_verifier.clone();
        let server = match &self.config.local_server {
            Some(config) => LocalServer::from_config(config.clone())?,
            None => LocalServer::new(self.config.redirect_uri.clone())?,
        };
        let listener = server.bind()?;

        let handle = tokio::task::spawn_blocking(move || server.listen_with(listener));

        on_authorize(&auth)?;

        let response = handle.await.map_err(|err| OAuthError::InvalidResponse {
            message: err.to_string(),
            body: String::new(),
        })??;

        self.exchange_code(response, &code_verifier, Some(&expected_state))
            .await
    }

    pub async fn exchange_code(
        &self,
        response: AuthorizationResponse,
        code_verifier: &str,
        expected_state: Option<&str>,
    ) -> Result<TokenResponse, OAuthError> {
        let AuthorizationResponse { code, state } = response;
        let returned_state = state.as_deref();

        if let (Some(expected), Some(returned)) = (expected_state, returned_state) {
            if expected != returned {
                return Err(OAuthError::StateMismatch {
                    expected: expected.to_string(),
                    received: returned.to_string(),
                });
            }
        }

        let mut payload = HashMap::new();
        payload.insert("grant_type".to_string(), "authorization_code".to_string());
        payload.insert("code".to_string(), code);
        payload.insert("client_id".to_string(), self.config.client_id.clone());
        payload.insert("redirect_uri".to_string(), self.config.redirect_uri.clone());
        payload.insert("code_verifier".to_string(), code_verifier.to_string());

        if let Some(secret) = &self.config.client_secret {
            payload.insert("client_secret".to_string(), secret.clone());
        }

        if let Some(state_value) = returned_state.or(expected_state) {
            payload.insert("state".to_string(), state_value.to_string());
        }

        self.send_token_request(payload).await
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse, OAuthError> {
        let mut payload = HashMap::new();
        payload.insert("grant_type".to_string(), "refresh_token".to_string());
        payload.insert("refresh_token".to_string(), refresh_token.to_string());
        payload.insert("client_id".to_string(), self.config.client_id.clone());

        if let Some(secret) = &self.config.client_secret {
            payload.insert("client_secret".to_string(), secret.clone());
        }

        self.send_token_request(payload).await
    }

    async fn send_token_request(
        &self,
        mut payload: HashMap<String, String>,
    ) -> Result<TokenResponse, OAuthError> {
        for (key, value) in self.provider.token_params() {
            payload.insert(key, value);
        }
        for (key, value) in &self.config.token_params {
            payload.insert(key.clone(), value.clone());
        }

        let headers = self.provider.token_headers();
        let mut builder = self.http.post(self.provider.token_url());
        builder = apply_headers(builder, &headers)?;

        let response = match self.provider.token_request_format() {
            TokenRequestFormat::Json => builder.json(&payload).send().await?,
            TokenRequestFormat::Form => builder.form(&payload).send().await?,
        };

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(OAuthError::HttpStatus {
                status: status.as_u16(),
                body,
            });
        }

        let token = serde_json::from_str(&body).map_err(|err| OAuthError::InvalidResponse {
            message: err.to_string(),
            body,
        })?;

        Ok(token)
    }
}

fn apply_headers(
    mut builder: RequestBuilder,
    headers: &[(String, String)],
) -> Result<RequestBuilder, OAuthError> {
    for (name, value) in headers {
        let name =
            HeaderName::from_bytes(name.as_bytes()).map_err(|_| OAuthError::InvalidHeader {
                name: name.clone(),
                value: value.clone(),
            })?;
        let value = HeaderValue::from_str(value).map_err(|_| OAuthError::InvalidHeader {
            name: name.to_string(),
            value: value.clone(),
        })?;
        builder = builder.header(name, value);
    }
    Ok(builder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AnthropicProvider;

    #[test]
    fn authorization_url_includes_required_params() {
        let config = OAuthClientConfig::new("client-id", "http://localhost:8765/callback");
        let client = OAuthClient::new(AnthropicProvider, config).unwrap();
        let auth = client.authorization_url().unwrap();

        let url = Url::parse(&auth.authorization_url).unwrap();
        let pairs: HashMap<_, _> = url.query_pairs().into_owned().collect();

        assert_eq!(pairs.get("response_type"), Some(&"code".to_string()));
        assert_eq!(pairs.get("client_id"), Some(&"client-id".to_string()));
        assert_eq!(
            pairs.get("redirect_uri"),
            Some(&"http://localhost:8765/callback".to_string())
        );
        assert_eq!(
            pairs.get("code_challenge_method"),
            Some(&"S256".to_string())
        );
        assert!(pairs.contains_key("code_challenge"));
        assert!(pairs.contains_key("state"));
        assert_eq!(pairs.get("code"), Some(&"true".to_string()));
    }
}
