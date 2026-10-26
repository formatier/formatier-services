IF (!record::exists($account_key)) {
    THROW 'account not found';
} ELSE {
    CREATE ONLY $user_key CONTENT $data;
};