select id, password
from backoffice_users
where username = :username
limit 1;