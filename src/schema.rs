table! {
    aircraft (id) {
        id -> Int4,
        user_id -> Int4,
        model -> Varchar,
        manufacturer -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Text,
    }
}

joinable!(aircraft -> users (user_id));

allow_tables_to_appear_in_same_query!(
    aircraft,
    users,
);
