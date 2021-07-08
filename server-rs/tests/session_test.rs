#![feature(async_closure)]
#[macro_use]
extern crate diesel_migrations;


mod common;

use common::session_test;

#[tokio::test(flavor = "multi_thread")]
async fn test_version() {
    session_test(r#"
[C1] version 2
[S1] okay
    "#).await;
    session_test(r#"
[C1] version 1
    "#).await;
    session_test(r#"
[C1] version 3
[S1] error invalid protocol version
    "#).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user() {
    session_test(r#"
[C1] version 2
[S1] okay
[C1] new_tmp_user Test
[S1] okay
[C1] self_user_info
[S1] self_user_info 1, Test, -
[C1] logout
[S1] okay
[C1] self_user_info
[S1] error you are not logged in
[C2] version 2
[S2] okay
[C2] new_tmp_user Test2
[S2] okay
[C2] self_user_info
[S2] self_user_info 2, Test2, -
    "#).await;

    session_test(r#"
[C1] version 2
[S1] okay
[C1] new_user Test, test@example.com, password
[S1] okay
[C1] self_user_info
[S1] self_user_info 1, Test, test@example.com
[C2] version 2
[S2] okay
[C2] login test@example.com, password
[S2] okay
[C2] self_user_info
[S2] self_user_info 1, Test, test@example.com
[C2] login test@example.com, random
[S2] error incorrect login credentials
    "#).await;
}
