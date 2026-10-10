use axum::body::Body;
use axum::http::{Request, StatusCode};
use sefin_mock::compression::utf8_to_gzip_b64;
use tower::ServiceExt;

mod support;

#[tokio::test]
async fn post_nfse_then_get_dps() {
    let app = support::test_app();
    let dps_xml = r#"<?xml version="1.0"?><DPS><infDPS Id="DPSMOCK001"><Signature/></infDPS></DPS>"#;
    let b64 = utf8_to_gzip_b64(dps_xml).unwrap();
    let body = serde_json::json!({ "dpsXmlGZipB64": b64 });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/SefinNacional/nfse")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let get = app
        .oneshot(
            Request::builder()
                .uri("/SefinNacional/dps/DPSMOCK001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get.status(), StatusCode::OK);
}
