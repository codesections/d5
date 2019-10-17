use base64;
use std::{collections::HashMap, env, net, sync::Arc, sync::RwLock};
use warp::{header, reply::with_status, Filter};
use warp::{http::StatusCode as Code, reject::custom as warp_err};

type WarpResult = Result<String, warp::Rejection>;
type DB = Arc<RwLock<HashMap<String, String>>>;
use crate::Err::{Db, NotFound, Unauthorized};

fn main() {
    // Configuration via env variables
    let port = env::var("PORT").unwrap_or_default().parse().unwrap_or(3030);
    let addr = env::var("HOST")
        .unwrap_or_default()
        .parse()
        .unwrap_or_else(|_| net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)));

    // Optional key for single-user mode; `USER:PASSWORD`
    //    We must base64-encode the key and prefix it with `Basic ` to match curl's format
    let prefix = |s| format!("Basic {}", s);
    let key = env::var("KEY").map(|s| base64::encode(&s)).map(prefix).ok();
    let key = warp::any().map(move || key.clone());

    // Store all IP addresses in a thread-safe hash map
    let db: DB = Arc::new(RwLock::new(HashMap::new()));
    let db = warp::any().map(move || db.clone());

    let get = warp::get2()
        .and(header("authorization"))
        .and(db.clone())
        .and_then(move |id: String, ip: DB| -> WarpResult {
            match ip.read().map_err(|_| warp_err(Db))?.get(&id) {
                Some(ip) => Ok(ip.to_string()),
                None => Err(warp::reject::custom(NotFound)),
            }
        });

    let post = warp::post2()
        .and(header("X-Forwarded-For").or(header("remote_addr")).unify())
        .and(warp::header::<String>("authorization"))
        .and(db.clone())
        .and(key.clone())
        .and_then(move |ip: String, id: String, db: DB, key: Option<String>| {
            if key.is_some() && key.unwrap() != id {
                return Err(warp_err(Unauthorized));
            }
            db.write().map_err(|_| warp_err(Db))?.insert(id, ip.clone());
            Ok(ip)
        });

    let delete = warp::delete2()
        .and(header("authorization"))
        .and(db)
        .and_then(move |id: String, db: DB| -> WarpResult {
            match db.write().map_err(|_| warp_err(Db))?.remove(&id) {
                Some(_) => Ok("IP deleted".to_string()),
                None => Err(warp_err(NotFound)),
            }
        });

    let handle_err = |err: warp::Rejection| match err.find_cause::<Err>() {
        Some(Db) => Ok(with_status(Db.to_string(), Code::INTERNAL_SERVER_ERROR)),
        Some(NotFound) => Ok(with_status(NotFound.to_string(), Code::NOT_FOUND)),
        Some(Unauthorized) => Ok(with_status(Unauthorized.to_string(), Code::UNAUTHORIZED)),
        None => Err(err),
    };

    eprintln!("d5 running on {}:{}", addr, port);
    warp::serve(get.or(post).or(delete).recover(handle_err)).run((addr, port));
}

#[derive(Debug)]
enum Err {
    Db,
    NotFound,
    Unauthorized,
}

impl std::fmt::Display for Err {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Db => "Internal server error.\n",
            Self::NotFound => "No IP found for that username–password pair.\n",
            Self::Unauthorized => "Unauthorized request.\n",
        })
    }
}
impl std::error::Error for Err {}
