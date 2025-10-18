PRAGMA foreign_keys = ON;

create table backoffice_users
(
    id       integer primary key autoincrement not null,
    username text unique                       not null,
    password blob                              not null,
    role     text                              not null
);

create table user_login_tokens
(
    user_id      integer     not null,
    token        text unique not null,
    expire_after text        not null,
    foreign key (user_id) references backoffice_users (id) on delete cascade
);

create table error_stack
(
    id            integer primary key autoincrement not null,
    error_name    text                              not null,
    error_summary text                              not null,
    error_stack   text                              not null,
    reported_at   text                              not null
);