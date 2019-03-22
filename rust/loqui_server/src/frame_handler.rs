use std::future::Future;

use loqui_protocol::codec::LoquiFrame;
use loqui_protocol::frames::{Error as ErrorFrame, Hello, HelloAck, Ping, Pong, Request, Response};
// TODO: rename this to request handler
use super::handler::{Handler, RequestContext};
use failure::Error;
use std::sync::Arc;

pub trait FrameHandler: Send + Sync + 'static {
    fn handle_frame(
        &self,
        frame: LoquiFrame,
    ) -> Box<dyn Future<Output = Result<Option<LoquiFrame>, Error>> + Send>;
}

pub struct ServerFrameHandler {
    inner: Arc<InnerHandler>,
}

impl ServerFrameHandler {
    pub fn handle_frame(
        &self,
        frame: LoquiFrame,
        // TODO: should we just return LoquiFrame::Error if there is an error??
    ) -> Box<dyn Future<Output = Result<Option<LoquiFrame>, Error>> + Send + 'static> {
        self.inner.clone().handle_frame(frame)
    }
}

pub struct InnerHandler {
    request_handler: Arc<dyn Handler>,
}

impl InnerHandler {
    pub fn handle_frame(
        &self,
        frame: LoquiFrame,
        // TODO: should we just return LoquiFrame::Error if there is an error??
    ) -> Box<dyn Future<Output = Result<Option<LoquiFrame>, Error>> + Send + 'static> {
        Box::new(
            async move {
                match frame {
                    LoquiFrame::Request(request) => await!(self.handle_request(request)),
                    LoquiFrame::Hello(hello) => {
                        Ok(None)
                        //self.handle_hello(hello)
                    },
                    LoquiFrame::Ping(ping) => {
                        Ok(None)
                        //Ok(Some(self.handle_ping(ping)))
                    },
                    frame => {
                        println!("unhandled frame {:?}", frame);
                        Ok(None)
                    }
                }
            }
        )
    }

    fn handle_ping(&self, ping: Ping) -> LoquiFrame {
        let Ping { flags, sequence_id } = ping;
        LoquiFrame::Pong(Pong { flags, sequence_id })
    }

    fn handle_hello(&self, hello: Hello) -> Result<Option<LoquiFrame>, Error> {
        let Hello {
            flags,
            version,
            encodings,
            compressions,
        } = hello;
        Ok(Some(LoquiFrame::HelloAck(HelloAck {
            flags,
            ping_interval_ms: 5000,
            encoding: encodings[0].clone(),
            compression: "".to_string(),
        })))
    }

    async fn handle_request(&self, request: Request) -> Result<Option<LoquiFrame>, Error> {
        let Request {
            payload,
            flags,
            sequence_id,
        } = request;

        let request_context = RequestContext {
            payload,
            flags,
            // TODO:
            encoding: "json".to_string(),
        };
        let result = await!(Box::into_pin(
            self.request_handler.handle_request(request_context)
        ));
        match result {
            Ok(payload) => Ok(Some(LoquiFrame::Response(Response {
                flags,
                sequence_id,
                payload,
            }))),
            Err(e) => {
                dbg!(e);
                // TODO:
                Ok(Some(LoquiFrame::Error(ErrorFrame {
                    flags,
                    sequence_id,
                    code: 0,
                    payload: vec![],
                })))
            }
        }
    }
}
