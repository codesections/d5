use argonautica::Hasher;
use base64;
use std::fs;
use warp::{http::StatusCode, reject::custom as warp_err, reply, Filter};

//TODO: Add: 1) comments; 2) env vars; 3) save to data dir
//      Don't add: more than 20 LoC
fn main() {
    let get = warp::get2()
        .and(warp::header::<HashedAuthHeader>("authorization"))
        .and_then(|filename: HashedAuthHeader| {
            fs::read_to_string(&filename.0).map_err(|_| warp_err(Error::NoSavedIp))
        });

    let post = warp::post2()
        .and(warp::header::<String>("X-Forwarded-For"))
        .and(warp::header::<HashedAuthHeader>("authorization"))
        .and_then(|ip: String, auth: HashedAuthHeader| {
            fs::write(&auth.0, ip.clone())
                .map(|_| ip)
                .map_err(|_| warp_err(Error::WriteErr))
        });

    let delete = warp::delete2()
        .and(warp::header::<HashedAuthHeader>("authorization"))
        .and_then(|filename: HashedAuthHeader| {
            fs::remove_file(&filename.0)
                .map(|_| "IP record deleted.\n")
                .map_err(|_| warp_err(Error::DeleteErr))
        });

    let handle_err = |err: warp::Rejection| {
        if let Some(&err) = err.find_cause::<Error>() {
            eprintln!("{}", err);
            Ok(reply::with_status(err.to_string(), StatusCode::FORBIDDEN))
        } else {
            Err(err)
        }
    };

    warp::serve(get.or(delete).or(post).recover(handle_err)).run(([127, 0, 0, 1], 3030));
}

struct HashedAuthHeader(String);
impl std::str::FromStr for HashedAuthHeader {
    type Err = warp::Rejection;
    fn from_str(auth_header: &str) -> Result<Self, Self::Err> {
        let without_prefix = auth_header["Basic ".len()..].to_string();
        Hasher::default()
            .opt_out_of_secret_key(true)
            .with_salt("fixed_salt")
            .with_password(without_prefix)
            .hash()
            .map(|h| Self(base64::encode(&h[32..])))
            .map_err(|_| warp_err(Error::CannotHash))
    }
}

#[derive(Debug, Copy, Clone)]
enum Error {
    NoSavedIp,
    WriteErr,
    CannotHash,
    DeleteErr,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::NoSavedIp => "No saved IP address matches that username/password pair.\n",
            Self::CannotHash => "Could not hash that username/password pair.\n",
            Self::WriteErr => "Could not write to the filesystem.\n",
            Self::DeleteErr => r"Could not delete IP address for that username/password pair.
Are you sure you used the correct pair?",
        })
    }
}
impl std::error::Error for Error {}
