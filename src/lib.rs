#![feature(impl_trait_in_assoc_type)]


use std::collections::HashMap;
use std::sync::Mutex;

use anyhow::{Error, anyhow};
use pilota::FastStr;


pub struct S {
    data: Mutex<HashMap<String, String>>,
}

impl S {
    pub fn new() -> Self {
        Self {
            data : Mutex::new(HashMap::new()),
        }
    }
}

#[volo::async_trait]
impl volo_gen::volo::example::ItemService for S {
	async fn get_item(&self, _req: volo_gen::volo::example::GetItemRequest)
     -> ::core::result::Result<volo_gen::volo::example::GetItemResponse, ::volo_thrift::AnyhowError> 
    {
        // so all the process here
        // tracing::info!("to set {:?} with {:?}", _req.key, _req.value);

        let mut res = volo_gen::volo::example::GetItemResponse {
            value: FastStr::from(""),
            stat: false,
        };

        if _req.ops == "set".to_string() {
            // tracing::info!("to set {:?} with {:?}", _req.key, _req.value);
            self.data.lock().unwrap().insert(_req.key.into_string(), _req.value.into_string());
            res.stat = true;
        } else if _req.ops == "get".to_string() {
            // tracing::info!("to get {:?}", _req.key);
            let k = _req.key.to_string();
            match self.data.lock().unwrap().get(&k) {
                Some(get_res) => {
                    res.stat = true;
                    res.value = FastStr::from(get_res.clone());
                }
                None => {
                    res.stat = false;
                }
            }
        } else if _req.ops == "del".to_string() {
            // tracing::info!("to del {:?}", _req.key);
            let k = _req.key.to_string();
            match self.data.lock().unwrap().remove(&k) {
                Some(ky) => {
                    tracing::info!("deled {:?}", _req.key);
                    res.stat = true;
                } 
                None => {
                    res.stat = false;
                }
            }

        } else if _req.ops == "ping".to_string() {
            // tracing::info!("to ping {:?}", _req.key);
            if _req.key.len() == 0 {
                res.value = FastStr::from("PONG");
            } else {
                res.value = FastStr::from(_req.key.clone());
            }
            res.stat = true;
        } else {
                // tracing::info!("invalid");
                return Err(Error::msg("invalid opcode"));
        }

        Ok(res)
    }
}


//////////////////////////////////////////////////// LogLayer

#[derive(Clone)]
pub struct LogService<S>(S);

#[volo::service]
impl<Cx, Req, S> volo::Service<Cx, Req> for LogService<S>
where
    Req: std::fmt::Debug + Send + 'static,
    S: Send + 'static + volo::Service<Cx, Req> + Sync,
    S::Response: std::fmt::Debug,
    S::Error: std::fmt::Debug,
    Cx: Send + 'static,
    anyhow::Error: Into<S::Error>,
{
    // type Req = volo_gen::volo::example::GetItemRequest;

    async fn call(&self, cx: &mut Cx, req: Req) -> Result<S::Response, S::Error> {
        let illegals = vec!["fword", "bword"];                              // filter example: fword / nword
        let r = format!("{:?}", &req);
        
        for ill in illegals {
            if r.contains(ill) {
                return Err(anyhow!("illegal words detected").into());                                   // contains the words to be filtered
            }
        }


        let now = std::time::Instant::now();
        // tracing::info!("Received request {:?}", &req);
        let resp = self.0.call(cx, req).await;
        // tracing::info!("Sent response {:?}", &resp);
        tracing::info!("Request took {}ms", now.elapsed().as_millis());
        resp
    }
}

pub struct LogLayer;

impl<S> volo::Layer<S> for LogLayer {
    type Service = LogService<S>;

    fn layer(self, inner: S) -> Self::Service {
        LogService(inner)
    }
}
