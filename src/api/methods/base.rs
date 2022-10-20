use std::collections::HashMap;

use reqwest::blocking::{Client, Response};
use reqwest::{Method};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::redirect::Policy;
use serde::{Deserialize};

use crate::api::Settings;
use crate::errors::Error;

#[derive(Deserialize, Debug)]
struct AuthData {
    pub userId: String,
    pub authToken: String,
}

#[derive(Deserialize, Debug)]
struct LoginResult {
    pub data: AuthData,
}

pub trait APIMethod {
    fn settings(&self) -> &Settings;
    fn endpoint(&self) -> &str;
    fn method(&self) -> Method;
    fn json_payload(&self) -> HashMap<&str, &str>;

    fn build_endpoint(&self, uri: &str) -> String {
        format!("{}{}", self.settings().domain, uri)
    }

    fn request(
        &self,
        endpoint: String,
        method: Method,
        json_map: &HashMap<&str, &str>,
        auth_data: Option<AuthData>,
    ) -> Result<Response, Error> {
        let mut headers = HeaderMap::new();
        if let Some(data) = &auth_data {
            let auth_token_hdr: &'static str = "x-auth-token";
            headers.insert(
                HeaderName::from_static(auth_token_hdr),
                HeaderValue::from_str(data.authToken.clone().as_str()).unwrap(),
            );

            let user_id_hdr: &'static str = "x-user-id";
            headers.insert(
                HeaderName::from_static(user_id_hdr),
                HeaderValue::from_str(data.userId.clone().as_str()).unwrap(),
            );
        }

        let mut request = Client::default()
            .request(method, endpoint)
            .headers(headers)
            .json(&json_map);

        match request.send() {
            Ok(response) => Ok(response),
            Err(err) => {
                let msg = format!("{}", err);
                Err(Error::RequestFailed(msg))
            }
        }
    }

    fn login_payload(&self) -> HashMap<&str, &str> {
        let mut payload = HashMap::new();
        payload.insert("user", self.settings().username.as_str());
        payload.insert("password", self.settings().password.as_str());

        payload
    }

    fn login(&self) -> Result<AuthData, Error> {
        let response = self.request(
            self.build_endpoint("/api/v1/login"),
            Method::POST,
            &self.login_payload(),
            None
        )?;

        if let Err(err) = response.error_for_status_ref() {
            let msg = format!("{}", err);
            return Err(Error::RequestFailed(msg));
        }

        let result: Result<LoginResult, _> = response.json();
        match result {
            Ok(login_result) => Ok(login_result.data),
            Err(err) => {
                let msg = format!("{}", err);
                Err(Error::JsonDecodeError(msg))
            }
        }
    }

    fn call(&self) -> Result<String, Error>{
        // TODO: add processing and return LoginError
        let auth_data= self.login()?;

        let response = self.request(
            self.build_endpoint(self.endpoint()),
            self.method(),
            &self.json_payload(),
            Some(auth_data)
        )?;

        if let Err(err) = response.error_for_status_ref() {
            let msg = format!("{}", err);
            return Err(Error::RequestFailed(msg));
        }

        match response.text() {
            Ok(text) => Ok(text),
            Err(_) => Err(Error::ResponseTextError),
        }
    }
}

pub trait Payload {
    fn json(&self) -> HashMap<&str, &str>;
}
