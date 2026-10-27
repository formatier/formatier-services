IF !record::exists($session_key) {
    THROW 'session not found';
} ELSE {
    DELETE FROM $session_key;
};