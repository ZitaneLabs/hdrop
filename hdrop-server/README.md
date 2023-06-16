## Migrate and generate new db schema

New table names:
File -> files

> Diesel assumes that your table name is the plural, snake-case form of your struct name. Because your table name does not follow this convention you can specify the table name explicitly

```bash
diesel migration run
diesel_ext > src/core/models.rs
```

Compilation:

Ubuntu:
install: libpq-dev

