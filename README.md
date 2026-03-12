# stash-lookup-rs

forward and reverse lookup for stashids

## api:
```
/health
  health endpoint

type can be any of
- performers
- studios
- tags

name can be alias or canonical name

/api/lookup/:type/:name
  look up by type and name
/api/id/:type/:id
  lookup by stashid only
```

## responses
tag:
```json
{
  "uuid": "00000000-0000-0000-0000-000000000000",
  "name": "Lorem Ipsum",
  "aliases": [
    "dolor sit amet",
    "consectetur adipiscing eli"
  ],
  "category": "00000000-0000-0000-0000-000000000000"
}
```
performer:
```json
{
  "uuid": "00000000-0000-0000-0000-000000000000",
  "name": "Lorem Ipsum",
  "aliases": [
    "dolor sit amet",
    "consectetur adipiscing eli"
  ]
}
```
studio:
```json
{
  "uuid": "00000000-0000-0000-0000-000000000000",
  "name": "Lorem Ipsum",
  "aliases": [
    "dolor sit amet",
    "consectetur adipiscing eli"
  ],
  "parent": "00000000-0000-0000-0000-000000000000"
}
```