// @generated automatically by Diesel CLI.

diesel::table! {
    accounts (id) {
        id -> Nullable<Text>,
        name -> Text,
        password -> Text,
    }
}

diesel::table! {
    alert_rules (id) {
        id -> Text,
        account_id -> Text,
        name -> Text,
        description -> Text,
        condition -> Text,
        severity -> Text,
        enabled -> Bool,
        created_at -> Text,
        updated_at -> Text,
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
        ip_address -> Nullable<Text>,
        hostname -> Nullable<Text>,
    }
}

diesel::table! {
    logs (id) {
        id -> Nullable<Integer>,
        account_id -> Text,
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

diesel::joinable!(alert_rules -> accounts (account_id));
diesel::joinable!(alerts -> alert_rules (rule_id));
diesel::joinable!(logs -> accounts (account_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    alert_rules,
    alerts,
    host,
    logs,
);
