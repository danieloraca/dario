use crate::header;
use dario_progress::{MAX_SAVE_BYTES, Progress, file};
use std::{
    io::{self, Read},
    path::Path,
};
use tiny_http::{Method, Request, Response};

fn error(request: Request, status: u16, message: &str) -> io::Result<()> {
    request.respond(
        Response::from_string(message)
            .with_status_code(status)
            .with_header(header("Cache-Control", "no-store")),
    )
}

pub fn respond(mut request: Request, directory: &Path) -> io::Result<()> {
    if request.headers().iter().any(|h| {
        h.field.equiv("Connection")
            && h.value
                .as_str()
                .split(',')
                .any(|value| value.trim().eq_ignore_ascii_case("upgrade"))
    }) {
        return error(request, 400, "Connection upgrades are not supported");
    }
    let profile = request
        .url()
        .split('?')
        .next()
        .unwrap_or("")
        .trim_start_matches("/api/progress/");
    if profile.is_empty()
        || profile.len() > 24
        || !profile
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
    {
        return error(
            request,
            400,
            "Use a player name of 1-24 lowercase letters, numbers, dashes or underscores.",
        );
    }
    let path = directory.join(format!("{profile}.json"));
    if !matches!(request.method(), Method::Get | Method::Put) {
        return request.respond(
            Response::from_string("Method not allowed")
                .with_status_code(405)
                .with_header(header("Allow", "GET, PUT")),
        );
    }
    let mut progress = match file::load(&path) {
        Ok(progress) => progress,
        Err(cause) => {
            eprintln!("Could not load {}: {cause}", path.display());
            return error(
                request,
                409,
                "This save cannot be read. It has been preserved; choose another player or restore your backup.",
            );
        }
    };
    if request.method() == &Method::Put {
        if !request.headers().iter().any(|h| {
            h.field.equiv("Content-Type")
                && h.value.as_str().split(';').next() == Some("application/json")
        }) {
            return error(request, 415, "Expected application/json");
        }
        let Some(length) = request.body_length() else {
            return error(request, 411, "Content-Length required");
        };
        if length > MAX_SAVE_BYTES {
            return error(request, 413, "Save is too large");
        }
        let mut bytes = Vec::with_capacity(length);
        request
            .as_reader()
            .take((MAX_SAVE_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        if bytes.len() > MAX_SAVE_BYTES {
            return error(request, 413, "Save is too large");
        }
        let incoming = match Progress::decode(&bytes) {
            Ok(progress) => progress,
            Err(_) => return error(request, 400, "Invalid or unsupported save"),
        };
        // Requests are handled serially, so another tab's update cannot
        // interleave with this read/merge/write on this server.
        progress.merge(&incoming);
        if let Err(cause) = file::save(&path, &progress) {
            eprintln!("Could not save {}: {cause}", path.display());
            return error(request, 503, "Could not save progress. Please retry.");
        }
    }
    request.respond(
        Response::from_data(progress.encode())
            .with_header(header("Content-Type", "application/json"))
            .with_header(header("Cache-Control", "no-store"))
            .with_header(header("X-Content-Type-Options", "nosniff")),
    )
}
