create table todos (
    id    bigserial primary key,
    title text      not null,
    done  boolean   not null default false
);
