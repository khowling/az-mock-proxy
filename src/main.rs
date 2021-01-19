use bytes::Buf;
use futures::stream::{self, StreamExt};
use hyper::client::HttpConnector;
use hyper::service::{make_service_fn, service_fn};
use hyper::{header, Body, Client, Method, Request, Response, Server, StatusCode};
use regex::Regex;
use serde_json::json;
use std::convert::Infallible;
use std::net::SocketAddr;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;

static INDEX: &[u8] = b"<a href=\"test.html\">test.html</a>";
//static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";
static POST_DATA: &str = r#"{"original": "data"}"#;
static URL: &str = "http://127.0.0.1:1337/json_api";

async fn client_request_response(client: &Client<HttpConnector>) -> Result<Response<Body>> {
    let req = Request::builder()
        .method(Method::POST)
        .uri(URL)
        .header(header::CONTENT_TYPE, "application/json")
        .body(POST_DATA.into())
        .unwrap();

    let web_res = client.request(req).await?;
    // Compare the JSON we sent (before) with what we received (after):
    let before = stream::once(async {
        Ok(format!(
            "<b>POST request body</b>: {}<br><b>Response</b>: ",
            POST_DATA,
        )
        .into())
    });
    let after = web_res.into_body();
    let body = Body::wrap_stream(before.chain(after));

    Ok(Response::new(body))
}

async fn api_post_response(req: Request<Body>) -> Result<Response<Body>> {
    // Aggregate the body...
    let whole_body = hyper::body::aggregate(req).await?;
    // Decode as JSON...
    let mut data: serde_json::Value = serde_json::from_reader(whole_body.reader())?;
    // Change the JSON...
    data["test"] = serde_json::Value::from("test_value");
    // And respond with the new JSON.
    let json = serde_json::to_string(&data)?;
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json))?;
    Ok(response)
}

async fn api_get_response(req: Request<Body>) -> Result<Response<Body>> {
    let uri_string = req.uri().to_string();
    //let parsed_url = Url::parse(&uri_string).unwrap();
    //let hash_query: std::collections::HashMap<_, _> =
    //    parsed_url.query_pairs().into_owned().collect();
    //    hash_query.get('a');
    //println!("url : {}", uri_string);
    let sku_res = Regex::new(r"sku=([^&]+)").unwrap();
    let sku = match sku_res.captures(&uri_string) {
        Some(p) => p.get(1).map_or("", |m| m.as_str()),
        None => "not_found",
    };

    //println!("url param : {}", param);

    let john = json!({
        "sku": sku,
        "stocklevel": 43
    });

    let json = john.to_string();

    let response = Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json))?;
    Ok(response)
}

async fn response_examples(
    req: Request<Body>,
    client: Client<HttpConnector>,
) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => Ok(Response::new(INDEX.into())),
        (&Method::GET, "/test.html") => client_request_response(&client).await,
        (&Method::POST, "/qpi/set") => api_post_response(req).await,
        (&Method::GET, "/api/stock") => api_get_response(req).await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(NOTFOUND.into())
                .unwrap())
        }
    }
}

#[tokio::main]
async fn main() {
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    // Share a `Client` with all `Service`s
    let client = Client::new();

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(move |_| {
        // Move a clone of `client` into the `service_fn`.
        let client = client.clone();

        async {
            // service_fn converts our function into a `Service`
            Ok::<_, Infallible>(service_fn(move |req| {
                // Clone again to ensure that client outlives this closure.
                response_examples(req, client.to_owned())
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
