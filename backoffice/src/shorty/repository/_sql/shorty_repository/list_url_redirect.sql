select ur.id, ur.url_path, ur.url_redirect, ur.created_at, bu.username, ur.created_by_user_id
from url_redirect as ur
         inner join backoffice_users bu on bu.id = ur.created_by_user_id
order by ur.id asc