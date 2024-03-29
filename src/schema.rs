#![allow(unused_imports)]

table! {
    use diesel::sql_types::*;
    use diesel_geography::sql_types::*;

    flights (id) {
        id -> Int4,
        number -> Nullable<Int4>,
        user_id -> Int4,
        glider_id -> Nullable<Int4>,
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
        created_at -> Timestamptz,
        hikeandfly -> Bool,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_geography::sql_types::*;

    gliders (id) {
        id -> Int4,
        user_id -> Int4,
        model -> Varchar,
        manufacturer -> Varchar,
        since -> Nullable<Date>,
        until -> Nullable<Date>,
        source -> Nullable<Text>,
        cost -> Nullable<Int4>,
        comment -> Nullable<Text>,
    }
}

table! {
    use diesel::sql_types::*;
    use diesel_geography::sql_types::*;

    igcs (flight_id) {
        flight_id -> Int4,
        data -> Bytea,
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
        last_glider_id -> Nullable<Int4>,
        email -> Text,
        signed_up -> Timestamptz,
        news_opt_in -> Bool,
    }
}

joinable!(flights -> gliders (glider_id));
joinable!(flights -> users (user_id));
joinable!(igcs -> flights (flight_id));
joinable!(locations -> users (user_id));

allow_tables_to_appear_in_same_query!(flights, gliders, igcs, locations, spatial_ref_sys, users,);
