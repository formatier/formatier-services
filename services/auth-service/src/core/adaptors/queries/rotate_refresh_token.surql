UPDATE ONLY $session_key SET
    update_at = time::now(),
    refresh_version = $expected_version + 1
WHERE refresh_version = $expected_version OR refresh_version = NONE
RETURN AFTER;