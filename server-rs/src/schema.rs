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
