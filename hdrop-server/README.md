# Hdrop Server

## Migrate and generate new db schema (hdrop-db)

```bash
diesel migration run
diesel_ext > src/core/models.rs
```

## Infos

> Diesel assumes that your table name is the plural, snake-case form of your struct name. When your table name does not follow this convention you can specify the table name explicitly

### Compilation:

#### On Ubuntu:
install `libpq-dev`
