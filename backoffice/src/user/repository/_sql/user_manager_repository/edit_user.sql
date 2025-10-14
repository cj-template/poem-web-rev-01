update backoffice_users
set username = :username,
    role     = :role
where id = :id