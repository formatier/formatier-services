IF !record::exists($account_key) {
    THROW 'account not found';
} ELSE IF $account_key.user != NONE {
    THROW 'account already initialized';
} ELSE {
    CREATE ONLY $user_key CONTENT $data;
    UPDATE ONLY $account_key SET
        user = $user_key,
        update_at = time::now();
};