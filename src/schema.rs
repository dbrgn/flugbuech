table! {
    aircraft (id) {
        id -> Int4,
        pilot_id -> Int4,
        name -> Varchar,
        brand -> Varchar,
    }
}

table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
    }
}

joinable!(aircraft -> users (pilot_id));

allow_tables_to_appear_in_same_query!(
    aircraft,
    users,
);
