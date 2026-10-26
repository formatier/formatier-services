IF !record::exists($account_key) {
    THROW "account not found";
} ELSE IF record::exists($provider_key) {
    THROW "provider already linked";
} ELSE {
    DELETE provider WHERE account = $account_key AND provider = $provider;
    CREATE $provider_key SET
        account = $account_key,
        provider = $provider,
        email_verified = FALSE,
        email = $email;
};