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

    session_test(r#"
[C1] version 2
[S1] okay
[C1] new_user Test, test@example.com, password
[S1] okay
[C1] self_user_info
[S1] self_user_info 1, Test, test@example.com
[C1] name Name
[S1] okay
[C1] password pass
[S1] okay
[C2] version 2
[S2] okay
[C2] login test@example.com, pass
[S2] okay
[C2] self_user_info
[S2] self_user_info 1, Name, test@example.com
    "#).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_create() {
    session_test(r#"
[C1] version 2
[S1] okay
[C2] version 2
[S2] okay
[C1] new_tmp_user Test1
[S1] okay
[C2] new_tmp_user Test2
[S2] okay
[C1] new_game chess
[S1] new_game 1
[C2] new_game chess
[S2] new_game 2
[C1] join_game 2
[S1] okay
[C2] join_game 2
[S2] okay
[C2] join_game 2
[S2] error you are already in that game
[C1] leave_game 2
[S1] okay
[C1] join_game 2
[S1] okay
[C1] start_game 2
[S1] error you aren't the owner of that game
[C2] start_game 2
[S2] go 2, chess, 0, 0, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S2] okay
[C1] leave_game 2
[S1] error that game has already started
    "#).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_observe() {
    session_test(r#"
[C1] version 2
[S1] okay
[C2] version 2
[S2] okay
[C1] new_tmp_user Test1
[S1] okay
[C1] new_game chess
[S1] new_game 1
[C2] observe_game 1
[S2] game 1, chess, 1, false, false, -, [], -
[C1] join_game 1
[S1] okay
[S2] game 1, chess, 1, false, false, -, [[1, 0]], -
[C1] leave_game 1
[S1] okay
[S2] game 1, chess, 1, false, false, -, [], -
[C2] stop_observe_game 1
[S2] okay
[C1] join_game 1
[S1] okay
    "#).await;
}