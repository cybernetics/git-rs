mod parse;

use self::parse::{parse_log, LogEntry};
use futures::{future, Future};
use state;
use std::str;
use tokio_process::CommandExt;
use types::DispatchFuture;
use util::git;
use util::transport::send_message;

#[derive(Debug, Serialize)]
#[serde(tag = "reason")]
pub enum ErrorReason {
    RepoPathNotSet,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum OutboundMessage {
    Success { log: Vec<LogEntry> },
    Error(ErrorReason),
}

pub fn dispatch(connection_state: state::Connection) -> DispatchFuture {
    use self::ErrorReason::RepoPathNotSet;
    use error::protocol::{Error, ProcessError::{Encoding, Failed, Parsing}};

    match connection_state.repo_path.clone() {
        Some(repo_path) => Box::new(
            git::new_command_with_repo_path(&repo_path)
                .arg("log")
                .output_async()
                .map_err(|_| Error::Process(Failed))
                .and_then(|output| match str::from_utf8(&output.stdout) {
                    Ok(output) => future::ok(String::from(output)),
                    Err(_) => future::err(Error::Process(Encoding)),
                })
                .and_then(|result| -> DispatchFuture {
                    if result.len() == 0 {
                        // TODO: more semantic error
                        return Box::new(future::err(Error::Process(Parsing)));
                    }

                    match parse_log(&format!("{}\n", result)) {
                        Ok((_, log)) => Box::new(send_message(
                            connection_state,
                            OutboundMessage::Success { log },
                        )),
                        Err(_) => Box::new(future::err(Error::Process(Parsing))),
                    }
                }),
        ),
        None => Box::new(send_message(
            connection_state,
            OutboundMessage::Error(RepoPathNotSet),
        )),
    }
}
