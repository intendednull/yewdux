use serde::{Deserialize, Serialize};
use std::rc::Rc;
use yew::prelude::*;
use yew_agent::reactor::{use_reactor_bridge, ReactorEvent, ReactorProvider};
use yewdux::{
    log::{log, Level},
    prelude::*,
};
use yewdux_idb::{load, DatabaseObjectPointer, IndexedDbReactor, QueueStatus, Request, Response};

/// Data object to send over the line.
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum Data {
    Counter(Rc<Counter>),
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Store)]
pub struct Counter {
    count: u32,
}

#[function_component]
pub fn App() -> Html {
    let (state, dispatch) = use_store::<Counter>();
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);

    yew::platform::spawn_local(async {
        match load::<Counter>(DatabaseObjectPointer::new(
            "idb-agent".to_string(),
            "counter".to_string(),
        ))
        .await
        {
            Ok(value) => {
                if let Some(v) = value {
                    log!(Level::Info, "got counter");
                    Dispatch::<Counter>::new().set(v);
                }
            }
            Err(e) => {
                log!(Level::Error, "{:?}", e);
            }
        }
    });

    html! {
        <ReactorProvider<IndexedDbReactor<Data>> path="./worker.js">
        <p>{ state.count }</p>
        <button {onclick}>{"+1"}</button>
        <IdbListener />
        </ReactorProvider<IndexedDbReactor<Data>>>
    }
}

#[function_component]
pub fn IdbListener() -> Html {
    let (state, _dispatch) = use_store::<Counter>();

    let status = use_state(|| QueueStatus::default());

    let reactor = {
        let status = status.setter();
        use_reactor_bridge::<IndexedDbReactor<Data>, _>(move |response| {
            if let ReactorEvent::Output(response) = response {
                match response {
                    Response::QueueStatus(s) => status.set(s),
                    Response::Error(e) => {
                        log!(Level::Error, "{:?}", e);
                    }
                };
            };
        })
    };

    use_effect_with((state.clone(), reactor.clone()), |(state, handle)| {
        handle.send(Request::put(
            "idb-agent".to_string(),
            "counter".to_string(),
            Data::Counter(state.clone()),
        ))
    });

    html! {
        <div>{format!{"{:?}", status}}</div>
    }
}
