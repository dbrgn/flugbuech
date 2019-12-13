#![allow(unused_imports)]

table! {
    use diesel::sql_types::*;
    use diesel_geography::sql_types::*;

    aircraft (id) {
        id -> Int4,
        user_id -> Int4,
        model -> Varchar,
        manufacturer -> Varchar,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_geography::sql_types::*;

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
        igc -> Nullable<Bytea>,
        created_at -> Timestamptz,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_geography::sql_types::*;

    locations (id) {
        id -> Int4,
        name -> Text,
        country -> Bpchar,
        elevation -> Int4,
        user_id -> Int4,
        geog -> Nullable<Geography>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_geography::sql_types::*;

    spatial_ref_sys (srid) {
        srid -> Int4,
        auth_name -> Nullable<Varchar>,
        auth_srid -> Nullable<Int4>,
        srtext -> Nullable<Varchar>,
        proj4text -> Nullable<Varchar>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_geography::sql_types::*;

    users (id) {
        id -> Int4,
        username -> Varchar,
        password -> Text,
        last_aircraft_id -> Nullable<Int4>,
        email -> Text,
    }
}

joinable!(flights -> aircraft (aircraft_id));
joinable!(flights -> users (user_id));
joinable!(locations -> users (user_id));

allow_tables_to_appear_in_same_query!(aircraft, flights, locations, spatial_ref_sys, users,);
