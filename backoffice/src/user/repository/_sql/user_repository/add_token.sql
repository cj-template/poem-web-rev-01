insert into user_login_tokens(user_id, token, expire_after)
values (:user_id, :token, datetime('now', '+30 day'))