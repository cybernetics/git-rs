mod status_entry;

use self::status_entry::{parse_git_status, StatusResult};
use config;
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
    Success { status: StatusResult },
    Error(ErrorReason),
}

pub fn dispatch(connection_state: state::Connection) -> DispatchFuture {
    use self::ErrorReason::RepoPathNotSet;
    use error::protocol::{Error, ProcessError::{Encoding, Failed}};

    match connection_state.repo_path.clone() {
        Some(repo_path) => Box::new(
            git::new_command_with_repo_path(&repo_path)
                .arg("status")
                .arg("--porcelain=v2")
                .arg("--untracked-files")
                .output_async()
                .then(|result| match result {
                    Ok(output) => match str::from_utf8(&output.stdout) {
                        Ok(output) => future::ok((String::from(output), connection_state)),
                        Err(_) => future::err((Error::Process(Encoding), connection_state)),
                    },
                    Err(_) => future::err((Error::Process(Failed), connection_state)),
                })
                .and_then(|(result, connection_state)| -> DispatchFuture {
                    if result.len() == 0 {
                        return Box::new(send_message(
                            connection_state,
                            OutboundMessage::Success { status: StatusResult::new() }
                        ));
                    }

                    match parse_git_status(&result) {
                        Ok(status) => Box::new(send_message(
                            connection_state,
                            OutboundMessage::Success { status },
                        )),
                        Err(e) => Box::new(future::err((e, connection_state))),
                    }
                }),
        ),
        None => Box::new(send_message(
            connection_state,
            OutboundMessage::Error(RepoPathNotSet),
        )),
    }
}
