table! {
    aircraft (id) {
        id -> Int4,
        user_id -> Int4,
        model -> Varchar,
        manufacturer -> Varchar,
    }
}

table! {
    flights (id) {
        id -> Int4,
        number -> Nullable<Int4>,
        user_id -> Int4,
        aircraft_id -> Nullable<Int4>,
        launch_at -> Nullable<Int4>,
        landing_at -> Nullable<Int4>,
        launch_time -> Nullable<Timestamptz>,
        landing_time -> Nullable<Timestamptz>,
        track_distance -> Nullable<Float4>,
        xcontest_tracktype -> Nullable<Text>,
        xcontest_distance -> Nullable<Float4>,
        xcontest_url -> Nullable<Text>,
        comment -> Nullable<Text>,
        video_url -> Nullable<Text>,
    }
}

table! {
    locations (id) {
        id -> Int4,
        name -> Text,
        country -> Bpchar,
        elevation -> Int4,
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
joinable!(flights -> aircraft (aircraft_id));
joinable!(flights -> users (user_id));

allow_tables_to_appear_in_same_query!(aircraft, flights, locations, users);
