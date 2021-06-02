table! {
    game_players (id) {
        id -> Int4,
        user_id -> Int4,
        game_id -> Int4,
        score -> Nullable<Float8>,
    }
}

table! {
    games (id) {
        id -> Int4,
        owner_id -> Int4,
        game_type -> Text,
        state -> Nullable<Text>,
        finished -> Bool,
        winner -> Nullable<Int4>,
        is_tie -> Nullable<Bool>,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Nullable<Text>,
        name -> Text,
        is_admin -> Bool,
        password_hash -> Nullable<Text>,
        api_key_hash -> Text,
    }
}

allow_tables_to_appear_in_same_query!(game_players, games, users,);
