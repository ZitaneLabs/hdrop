// @generated automatically by Diesel CLI.

diesel::table! {
    files (uuid) {
        uuid -> Uuid,
        accessToken -> Text,
        updateToken -> Text,
        dataUrl -> Nullable<Text>,
        fileNameData -> Text,
        salt -> Text,
        iv -> Text,
        createdAt -> Timestamptz,
        expiresAt -> Timestamptz,
        challengeData -> Text,
        challengeHash -> Text,
    }
}
