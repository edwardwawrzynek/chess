#[macro_use]
extern crate diesel_migrations;

mod common;

use common::session_test;

#[tokio::test(flavor = "multi_thread")]
async fn test_version() {
    session_test(
        r#"
[C1] version 2
[S1] okay
    "#,
    )
    .await;
    session_test(
        r#"
[C1] version 1
    "#,
    )
    .await;
    session_test(
        r#"
[C1] version 3
[S1] error invalid protocol version
    "#,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_user() {
    session_test(
        r#"
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
    "#,
    )
    .await;

    session_test(
        r#"
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
    "#,
    )
    .await;

    session_test(
        r#"
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
    "#,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_create() {
    session_test(
        r#"
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
    "#,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_observe() {
    session_test(
        r#"
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
    "#,
    )
    .await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_play() {
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
[C1] join_game 1
[S1] okay
[C2] join_game 1
[S2] okay
[C1] start_game 1
[S1] go 1, chess, 0, 0, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S1] okay
[C1] play 1, e2e4
[S1] okay
[S2] go 1, chess, 0, 0, rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
[C2] play 1, f7f6
[S2] okay
[S1] go 1, chess, 0, 0, rnbqkbnr/ppppp1pp/5p2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2
[C1] play 1, a2a3
[S1] okay
[S2] go 1, chess, 0, 0, rnbqkbnr/ppppp1pp/5p2/8/4P3/P7/1PPP1PPP/RNBQKBNR b KQkq - 0 2
[C2] play 1, g7g5
[S2] okay
[S1] go 1, chess, 0, 0, rnbqkbnr/ppppp2p/5p2/6p1/4P3/P7/1PPP1PPP/RNBQKBNR w KQkq g6 0 3
[C1] observe_game 1
[S1] game 1, chess, 1, true, false, -, [[1, 0], [2, 0]], rnbqkbnr/ppppp2p/5p2/6p1/4P3/P7/1PPP1PPP/RNBQKBNR w KQkq g6 0 3,[e2e4,f7f6,a2a3,g7g5]
[C1] play 1, d1h5
[S1] game 1, chess, 1, true, true, 1, [[1, 1], [2, 0]], rnbqkbnr/ppppp2p/5p2/6pQ/4P3/P7/1PPP1PPP/RNB1KBNR b KQkq - 0 3,[e2e4,f7f6,a2a3,g7g5,d1h5]
[S1] okay
[C2] version 2
[S2] okay
[C1] version 2
[S1] okay
    "#).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn test_game_protocol_versions() {
    session_test(
        r#"
// setup three clients, with C1 & C3 on the same user, but different versions
[C1] version 2
[S1] okay
[C2] version 2
[S2] okay
[C3] version 1
[C1] new_user Test, test@example.com, password
[S1] okay
[C2] new_tmp_user Test2
[S2] okay
[C3] login test@example.com, password
[C1] new_game chess
[S1] new_game 1
[C3] new_game chess
[S3] new_game 2
[C1] join_game 1
[S1] okay
[C2] join_game 1
[S2] okay
[C1] join_game 2
[S1] okay
[C2] join_game 2
[S2] okay
[C1] start_game 1
// both C1 & C3 get game 1
[S1] go 1, chess, 0, 0, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S3] board rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S1] okay
[C1] start_game 2
// only C1 gets game 2 (since C3 is in legacy, and only gets one game at a time)
[S1] go 2, chess, 0, 0, rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
[S1] okay
[C1] play 2, e2e4
[S1] okay
[S2] go 2, chess, 0, 0, rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
[C2] play 2, f7f6
[S2] okay
[S1] go 2, chess, 0, 0, rnbqkbnr/ppppp1pp/5p2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2
// move plays in oldest game (1)
[C3] move e2e4
[S2] go 1, chess, 0, 0, rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1
[C1] version 1
[C2] play 1, f7f6
[S2] okay
[S1] board rnbqkbnr/ppppp1pp/5p2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2
[S3] board rnbqkbnr/ppppp1pp/5p2/8/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2
[C1] play 1, a1a1
[S1] error that command is only available in protocol version 2 (you are in version 1)
    "#,
    )
    .await;
}
