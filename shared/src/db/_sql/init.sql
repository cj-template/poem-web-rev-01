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

create table url_redirect
(
    id                 integer primary key autoincrement not null,
    url_path           text unique                       not null,
    url_redirect       text unique                       not null,
    created_at         text                              not null,
    created_by_user_id integer                           not null,
    foreign key (created_by_user_id) references backoffice_users (id) on delete cascade
);

create table error_stack
(
    id            integer primary key autoincrement not null,
    error_name    text                              not null,
    error_summary text                              not null,
    error_stack   text                              not null,
    reported_at   text                              not null
);