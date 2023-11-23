use std::time::Duration;

use ::reqwest::redirect::Policy;
use anyhow::{anyhow, Ok, Result};
use reqwest;

pub struct CaptchaState {
    pub captcha_present: bool,
    pub session_id: String,
    pub message: Option<String>,
}

pub struct PortalClient {
    client: reqwest::Client,
    base_url: &'static str,
}

impl PortalClient {
    pub fn new(base_url: &'static str, user_agent: &'static str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .pool_idle_timeout(Duration::from_nanos(1)) // disable connection pooling
            .redirect(Policy::none()) // disable redirects
            .user_agent(user_agent)
            .http1_only()
            .build()?;

        Ok(PortalClient { client, base_url })
    }

    fn parse_page(&self, page: &str) -> Result<CaptchaState> {
        let dom = tl::parse(&page, tl::ParserOptions::default())?;
        let parser = dom.parser();

        let session_id = dom
            .query_selector("input[name=PHPSESSID]")
            .and_then(|mut x| x.next())
            .and_then(|x| x.get(parser))
            .and_then(|x| x.as_tag())
            .and_then(|x| x.attributes().get("value").flatten())
            .and_then(|x| x.try_as_utf8_str())
            .ok_or(anyhow!("PHPSESSID not found"))?
            .to_string();

        let captcha = dom.get_element_by_id("captcha").is_some();

        let message = dom
            .query_selector("TD[align=center]")
            .and_then(|x| {
                // HACK: https://github.com/y21/tl/issues/22
                x.filter_map(|e| e.get(parser).and_then(|e| e.children()))
                    .flat_map(|e| e.all(parser))
                    .filter_map(|e| e.as_tag())
                    .filter(|e| e.name() == "DIV")
                    .next()
            })
            .and_then(|x| Some(x.inner_text(parser).trim().to_string()));

        Ok(CaptchaState {
            captcha_present: captcha,
            session_id,
            message,
        })
    }

    pub async fn get_state(&self) -> Result<CaptchaState> {
        let page = self
            .client
            .post(self.base_url)
            .form(&[("viewForm", "true")])
            .send()
            .await?
            .text()
            .await?;

        self.parse_page(&page)
    }

    pub async fn get_captcha(&self, session_id: &str) -> Result<Vec<u8>> {
        let bytes = self
            .client
            .get(format!(
                "{}getCaptcha.html?PHPSESSID={}",
                self.base_url, session_id
            ))
            .send()
            .await?
            .bytes()
            .await?;

        Ok(bytes.to_vec())
    }

    pub async fn submit_captcha(&self, session_id: &str, solution: &str) -> Result<CaptchaState> {
        let page = self
            .client
            .post(self.base_url)
            .form(&[
                ("PHPSESSID", session_id),
                ("viewForm", "true"),
                ("captcha", solution),
            ])
            .send()
            .await?
            .text()
            .await?;

        self.parse_page(&page)
    }
}
