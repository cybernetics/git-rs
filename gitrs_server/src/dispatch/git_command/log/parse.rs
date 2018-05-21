use nom;
use util::parse::sha;

#[derive(Debug, Serialize)]
pub struct LogEntry {
    commit_sha: String,
    author: String,
    date: String,
    commit_message: String,
}

/*
* Each indent starts with 4 spaces
* seperator line with no spaces
*/
named!(pub parse_log_entry<&str, LogEntry>,
    do_parse!(
        tag!("commit ") >>
        commit_sha: sha >>
        char!('\n') >>
        tag!("Author: ") >>
        author: take_until!("\n") >>
        char!('\n') >>
        tag!("Date:   ") >>
        date: take_until!("\n") >>
        tag!("\n\n    ") >>
        commit_message: take_until!("\n\n") >>
        (LogEntry {
            commit_sha: String::from(commit_sha),
            author: String::from(author),
            date: String::from(date),
            commit_message: String::from(commit_message),
        })
    )
);

named!(pub parse_log<&str, Vec<LogEntry>>,
    do_parse!(
        entries: separated_list!(
            tag!("\n\n"),
            complete!(parse_log_entry)
        ) >>
        (entries)
    )
);
