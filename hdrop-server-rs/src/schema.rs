// @generated automatically by Diesel CLI.

diesel::table! {
    files (uuid) {
        id -> Int4,
        uuid -> Text,
        accessToken -> Text,
        updateToken -> Text,
        dataUrl -> Nullable<Text>,
        fileNameData -> Text,
        fileNameHash -> Text,
        salt -> Text,
        iv -> Text,
        createdAt -> Timestamp,
        expiresAt -> Timestamp,
    }
}

diesel::table! {
    statistics (id) {
        id -> Int4,
        uploadCount -> Int4,
        deleteCount -> Int4,
        existCount -> Int4,
    }
}

diesel::allow_tables_to_appear_in_same_query!(files, statistics,);
