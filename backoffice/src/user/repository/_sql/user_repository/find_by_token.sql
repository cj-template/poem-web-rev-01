select u.id, u.username, u.role
from backoffice_users as u
         inner join user_login_tokens ult on u.id = ult.user_id
where ult.token = :token
  and ult.expire_after > datetime('now')
limit 1;