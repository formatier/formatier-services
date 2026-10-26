IF !record::exists($account_key) {
    THROW 'account not found';
} ELSE {
    DELETE provider WHERE account = $account_key AND provider = $provider;
}