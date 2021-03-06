create table issue618 (id bigint not null primary key);
insert into issue618 (id) values (1);
update issue618 set id = id where id = 1;
create index idxissue618 on issue618 using zombodb ((issue618.*));
select ctid, * from issue618 where issue618 ==> 'zdb_ctid:1'; -- comes back from (0, 2) b/c it's a HOT, so the ctid doesn't match zdb_ctid
update issue618 set id = id where id = 1;
select ctid, * from issue618 where issue618 ==> 'zdb_ctid:3'; -- comes back as (0, 3)  b/c it's no longer HOT and ctid mathes zdb_ctid
delete from issue618 where id = 1;
drop table issue618;