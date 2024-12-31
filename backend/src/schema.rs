// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Text,
        name -> Text,
        password -> Text,
        role -> Text,
    }
}

diesel::table! {
    alerts (id) {
        id -> Text,
        rule_id -> Text,
        account_id -> Text,
        severity -> Text,
        message -> Text,
        acknowledged -> Bool,
        created_at -> Text,
    }
}

diesel::table! {
    host (id) {
        id -> Text,
        account_id -> Text,
        ip_address -> Nullable<Text>,
        hostname -> Nullable<Text>,
    }
}

diesel::table! {
    logs (id) {
        id -> Text,
        account_id -> Text,
        host_id -> Text,
        version -> Nullable<Text>,
        device_vendor -> Nullable<Text>,
        device_product -> Nullable<Text>,
        device_version -> Nullable<Text>,
        signature_id -> Nullable<Text>,
        name -> Nullable<Text>,
        severity -> Nullable<Text>,
        extensions -> Nullable<Text>,
    }
}

diesel::table! {
    rules (id) {
        id -> Text,
        account_id -> Text,
        title -> Text,
        status -> Text,
        description -> Text,
        references -> Text,
        tags -> Text,
        author -> Text,
        date -> Text,
        logsource -> Text,
        detection -> Text,
        fields -> Text,
        falsepositives -> Text,
        level -> Text,
        enabled -> Bool,
        created_at -> Text,
        updated_at -> Text,
    }
}

diesel::joinable!(alerts -> rules (rule_id));
diesel::joinable!(host -> accounts (account_id));
diesel::joinable!(logs -> accounts (account_id));
diesel::joinable!(logs -> host (host_id));
diesel::joinable!(rules -> accounts (account_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    alerts,
    host,
    logs,
    rules,
);
