use crate::Err::{Lock, NotFound, Unauthorized};
use base64;
use std::{collections::HashMap, env, net, sync::Arc, sync::Mutex};
use warp::{http::StatusCode as Code, reject::custom as warp_err, reply::with_status, Filter};

fn main() {
    type DB = Arc<Mutex<HashMap<String, String>>>;
    type WarpResult = Result<String, warp::Rejection>;

    let port = env::var("PORT").unwrap_or_default().parse().unwrap_or(3030);
    let addr = env::var("HOST")
        .unwrap_or_default()
        .parse()
        .unwrap_or_else(|_| net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)));

    let prefix = |s| format!("Basic {}", s);
    let key = env::var("KEY").map(|s| base64::encode(&s)).map(prefix).ok();
    let key = warp::any().map(move || key.clone());

    let db: DB = Arc::new(Mutex::new(HashMap::new()));
    let db = warp::any().map(move || db.clone());

    let get = warp::get2()
        .and(warp::header::<String>("authorization"))
        .and(db.clone())
        .and_then(move |id: String, ip: DB| -> WarpResult {
            match ip.lock().map_err(|_| warp_err(Lock))?.get(&id) {
                Some(v) => Ok(v.to_string()),
                None => Err(warp::reject::custom(NotFound)),
            }
        });

    let post = warp::post2()
        .and(warp::header::<String>("X-Forwarded-For"))
        .and(warp::header::<String>("authorization"))
        .and(db.clone())
        .and(key.clone())
        .and_then(move |ip: String, id: String, db: DB, key: Option<String>| {
            dbg!(&key, &id);
            if key.is_some() && key.unwrap() != id {
                return Err(warp_err(Err::Unauthorized));
            }
            db.lock().map_err(|_| warp_err(Lock))?.insert(id, ip);
            Ok("IP saved.\n".to_string())
        });

    let delete = warp::delete2()
        .and(warp::header::<String>("authorization"))
        .and(db)
        .and_then(move |id: String, db: DB| -> WarpResult {
            match db.lock().map_err(|_| warp_err(Lock))?.remove(&id) {
                Some(_) => Ok("IP deleted".to_string()),
                None => Err(warp_err(NotFound)),
            }
        });

    let handle_err = |err: warp::Rejection| match err.find_cause::<Err>() {
        Some(Lock) => Ok(with_status(Lock.to_string(), Code::INTERNAL_SERVER_ERROR)),
        Some(NotFound) => Ok(with_status(NotFound.to_string(), Code::NOT_FOUND)),
        Some(Unauthorized) => Ok(with_status(Unauthorized.to_string(), Code::UNAUTHORIZED)),
        None => Err(err),
    };

    eprintln!("d5 running on {}:{}", addr, port);
    warp::serve(get.or(post).or(delete).recover(handle_err)).run((addr, port));
}

#[derive(Debug)]
enum Err {
    Lock,
    NotFound,
    Unauthorized,
}

impl std::fmt::Display for Err {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            Self::Lock => "Internal server error.\n",
            Self::NotFound => "No IP found for that username/password pair.\n",
            Self::Unauthorized => "Unauthorized request.",
        })
    }
}
impl std::error::Error for Err {}
