use reqwasm::http::Request;
use serde_json::Value;
use std::collections::HashMap;

use yewdux::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status {
    Loading,
    Ready,
    Error,
}

#[derive(Default, Clone, PartialEq, Eq, Store)]
pub struct State {
    timezones: HashMap<String, (String, Status)>,
}

impl State {
    pub fn get(&self, timezone: &str) -> Option<(String, Status)> {
        self.timezones.get(timezone).cloned()
    }

    pub fn timezones(&self) -> impl Iterator<Item = String> + '_ {
        self.timezones.keys().cloned()
    }

    pub fn add(&mut self, timezone: String) {
        self.timezones
            .insert(timezone.clone(), ("...".into(), Status::Loading));

        self.refresh(timezone);
    }

    pub fn refresh(&mut self, timezone: String) {
        if let Some(e) = self.timezones.get_mut(&timezone) {
            e.1 = Status::Loading;
        }

        yew::platform::spawn_local(async move {
            let dispatch = Dispatch::<State>::global();
            let response = {
                let url = "http://worldtimeapi.org/api/timezone/".to_string() + timezone.as_str();
                Request::get(&url).send().await
            };

            match response {
                Ok(resp) if resp.ok() => {
                    let resp: Value = {
                        resp.text()
                            .await
                            .ok()
                            .and_then(|x| serde_json::from_str(&x).ok())
                            .expect("unexpected response")
                    };
                    let datetime = resp["datetime"].to_string();

                    dispatch.reduce_mut(|state| {
                        state.timezones.insert(timezone, (datetime, Status::Ready))
                    });
                }
                Ok(resp) => {
                    dispatch.reduce_mut(|state| {
                        state
                            .timezones
                            .insert(timezone, (resp.status_text(), Status::Error))
                    });
                }
                Err(e) => {
                    dispatch.reduce_mut(|state| {
                        state
                            .timezones
                            .insert(timezone, (e.to_string(), Status::Error))
                    });
                }
            }
        });
    }

    pub fn delete(&mut self, timezone: &str) {
        self.timezones.remove(timezone);
    }
}
