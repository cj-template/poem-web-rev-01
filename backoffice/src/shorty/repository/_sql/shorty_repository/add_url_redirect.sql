insert into url_redirect (url_path, url_redirect, created_at, created_by_user_id)
values (:url_path, :url_redirect, datetime(), :user_id);