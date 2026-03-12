use crate::models::{PerformerResponse, StudioResponse, TagResponse};
use sqlx::SqlitePool;
use uuid::Uuid;

fn split_names(rows: Vec<(String, i32)>) -> (String, Vec<String>) {
    let mut canonical = String::new();
    let mut aliases = Vec::with_capacity(rows.len().saturating_sub(1));
    for (name, role) in rows {
        if role == 0 {
            canonical = name;
        } else {
            aliases.push(name);
        }
    }
    (canonical, aliases)
}

fn placeholders(n: usize) -> String {
    vec!["?"; n].join(", ")
}

fn normalize_search(s: &str) -> Option<String> {
    let n = s.trim().to_lowercase();
    if n.is_empty() { None } else { Some(n) }
}

// performers
async fn performer_uuids_by_name(pool: &SqlitePool, search_name: &str) -> Result<Vec<Uuid>, sqlx::Error> {
    let Some(normalized) = normalize_search(search_name) else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query_as::<_, (Uuid,)>(
        r#"SELECT uuid FROM performer_names WHERE LOWER(TRIM(name)) = ? ORDER BY uuid"#,
    )
    .bind(&normalized)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(u,)| u).collect())
}

pub async fn lookup_performers_by_name(pool: &SqlitePool, search_name: &str) -> Result<Vec<PerformerResponse>, sqlx::Error> {
    let uuids = performer_uuids_by_name(pool, search_name).await?;
    if uuids.is_empty() {
        return Ok(Vec::new());
    }
    let ph = placeholders(uuids.len());
    let sql = format!(
        "SELECT uuid, name, role FROM performer_names WHERE uuid IN ({}) ORDER BY uuid, role",
        ph
    );
    let mut query = sqlx::query_as::<_, (Uuid, String, i32)>(&sql);
    for u in &uuids {
        query = query.bind(u);
    }
    let rows = query.fetch_all(pool).await?;
    let mut by_uuid: std::collections::HashMap<Uuid, Vec<(String, i32)>> = std::collections::HashMap::new();
    for (uuid, name, role) in rows {
        by_uuid.entry(uuid).or_default().push((name, role));
    }
    let mut results = Vec::with_capacity(by_uuid.len());
    for uuid in uuids {
        if let Some(names) = by_uuid.remove(&uuid) {
            let (canonical, aliases) = split_names(names);
            results.push(PerformerResponse { uuid, name: canonical, aliases });
        }
    }
    Ok(results)
}

pub async fn lookup_performer_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<PerformerResponse>, sqlx::Error> {
    let Some(uuid) = Uuid::parse_str(id).ok() else {
        return Ok(None);
    };

    let names = sqlx::query_as::<_, (String, i32)>(
        r#"
        SELECT name, role
        FROM performer_names
        WHERE uuid = ?
        ORDER BY role
        "#,
    )
    .bind(uuid)
    .fetch_all(pool)
    .await?;

    if names.is_empty() {
        return Ok(None);
    }

    let (canonical, aliases) = split_names(names);
    Ok(Some(PerformerResponse {
        uuid,
        name: canonical,
        aliases,
    }))
}

pub async fn add_performer(pool: &SqlitePool, uuid: &Uuid, name: &str, aliases: &[String]) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO performer_names (uuid, name, role)
        VALUES (?, ?, 0)
        ON CONFLICT(uuid, name) DO UPDATE SET role=0
        "#,
    )
    .bind(uuid)
    .bind(name.trim())
    .execute(pool)
    .await?;
    if !aliases.is_empty() {
        let values = aliases.iter().map(|_| "(?, ?, 1)").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "INSERT INTO performer_names (uuid, name, role) VALUES {} ON CONFLICT(uuid, name) DO UPDATE SET role=1",
            values
        );
        let mut query = sqlx::query(&sql);
        for alias in aliases {
            query = query.bind(uuid).bind(alias.trim());
        }
        query.execute(pool).await?;
    }
    Ok(())
}

// studios
async fn studio_uuids_by_name(pool: &SqlitePool, search_name: &str) -> Result<Vec<Uuid>, sqlx::Error> {
    let Some(normalized) = normalize_search(search_name) else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query_as::<_, (Uuid,)>(
        r#"SELECT studio_uuid FROM studio_names WHERE LOWER(TRIM(name)) = ? ORDER BY studio_uuid"#,
    )
    .bind(&normalized)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(u,)| u).collect())
}

pub async fn lookup_studios_by_name(pool: &SqlitePool, search_name: &str) -> Result<Vec<StudioResponse>, sqlx::Error> {
    let uuids = studio_uuids_by_name(pool, search_name).await?;
    if uuids.is_empty() {
        return Ok(Vec::new());
    }
    let ph = placeholders(uuids.len());
    let names_sql = format!(
        "SELECT studio_uuid, name, role FROM studio_names WHERE studio_uuid IN ({}) ORDER BY studio_uuid, role",
        ph
    );
    let mut names_query = sqlx::query_as::<_, (Uuid, String, i32)>(&names_sql);
    for u in &uuids {
        names_query = names_query.bind(u);
    }
    let name_rows = names_query.fetch_all(pool).await?;
    let parents_sql = format!(
        "SELECT uuid, parent FROM studios WHERE uuid IN ({})",
        ph
    );
    let mut parents_query = sqlx::query_as::<_, (Uuid, Option<Uuid>)>(&parents_sql);
    for u in &uuids {
        parents_query = parents_query.bind(u);
    }
    let parent_rows: std::collections::HashMap<Uuid, Option<Uuid>> =
        parents_query.fetch_all(pool).await?.into_iter().collect();
    let mut by_uuid: std::collections::HashMap<Uuid, Vec<(String, i32)>> = std::collections::HashMap::new();
    for (uuid, name, role) in name_rows {
        by_uuid.entry(uuid).or_default().push((name, role));
    }
    let mut results = Vec::with_capacity(by_uuid.len());
    for uuid in uuids {
        if let Some(names) = by_uuid.remove(&uuid) {
            let (canonical, aliases) = split_names(names);
            let parent = parent_rows.get(&uuid).copied().flatten();
            results.push(StudioResponse { uuid, parent, name: canonical, aliases });
        }
    }
    Ok(results)
}

pub async fn lookup_studio_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<StudioResponse>, sqlx::Error> {
    let Some(uuid) = Uuid::parse_str(id).ok() else {
        return Ok(None);
    };

    let names = sqlx::query_as::<_, (String, i32)>(
        r#"
        SELECT name, role
        FROM studio_names
        WHERE studio_uuid = ?
        ORDER BY role
        "#,
    )
    .bind(uuid)
    .fetch_all(pool)
    .await?;

    if names.is_empty() {
        return Ok(None);
    }

    let (parent,) = sqlx::query_as::<_, (Option<Uuid>,)>(
        r#"SELECT parent FROM studios WHERE uuid = ?"#,
    )
    .bind(uuid)
    .fetch_one(pool)
    .await?;

    let (canonical, aliases) = split_names(names);
    Ok(Some(StudioResponse {
        uuid,
        parent,
        name: canonical,
        aliases,
    }))
}

pub async fn add_studio(pool: &SqlitePool, uuid: &Uuid, name: &str, aliases: &[String], parent: Option<Uuid>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO studios (uuid, parent)
        VALUES (?, ?)
        ON CONFLICT(uuid) DO UPDATE SET parent=excluded.parent
        "#,
    )
    .bind(uuid)
    .bind(parent)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO studio_names (studio_uuid, name, role)
        VALUES (?, ?, 0)
        ON CONFLICT(studio_uuid, name) DO UPDATE SET role=0
        "#,
    )
    .bind(uuid)
    .bind(name.trim())
    .execute(pool)
    .await?;

    if !aliases.is_empty() {
        let values = aliases.iter().map(|_| "(?, ?, 1)").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "INSERT INTO studio_names (studio_uuid, name, role) VALUES {} ON CONFLICT(studio_uuid, name) DO UPDATE SET role=1",
            values
        );
        let mut query = sqlx::query(&sql);
        for alias in aliases {
            query = query.bind(uuid).bind(alias.trim());
        }
        query.execute(pool).await?;
    }
    Ok(())
}

// tags
async fn tag_uuids_by_name(pool: &SqlitePool, search_name: &str) -> Result<Vec<Uuid>, sqlx::Error> {
    let Some(normalized) = normalize_search(search_name) else {
        return Ok(Vec::new());
    };
    let rows = sqlx::query_as::<_, (Uuid,)>(
        r#"SELECT tag_uuid FROM tag_names WHERE LOWER(TRIM(name)) = ? ORDER BY tag_uuid"#,
    )
    .bind(&normalized)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(u,)| u).collect())
}

pub async fn lookup_tags_by_name(pool: &SqlitePool, search_name: &str) -> Result<Vec<TagResponse>, sqlx::Error> {
    let uuids = tag_uuids_by_name(pool, search_name).await?;
    if uuids.is_empty() {
        return Ok(Vec::new());
    }
    let ph = placeholders(uuids.len());
    let names_sql = format!(
        "SELECT tag_uuid, name, role FROM tag_names WHERE tag_uuid IN ({}) ORDER BY tag_uuid, role",
        ph
    );
    let mut names_query = sqlx::query_as::<_, (Uuid, String, i32)>(&names_sql);
    for u in &uuids {
        names_query = names_query.bind(u);
    }
    let name_rows = names_query.fetch_all(pool).await?;
    let categories_sql = format!(
        "SELECT uuid, category FROM tags WHERE uuid IN ({})",
        ph
    );
    let mut categories_query = sqlx::query_as::<_, (Uuid, Option<Uuid>)>(&categories_sql);
    for u in &uuids {
        categories_query = categories_query.bind(u);
    }
    let category_rows: std::collections::HashMap<Uuid, Option<Uuid>> =
        categories_query.fetch_all(pool).await?.into_iter().collect();
    let mut by_uuid: std::collections::HashMap<Uuid, Vec<(String, i32)>> = std::collections::HashMap::new();
    for (uuid, name, role) in name_rows {
        by_uuid.entry(uuid).or_default().push((name, role));
    }
    let mut results = Vec::with_capacity(by_uuid.len());
    for uuid in uuids {
        if let Some(names) = by_uuid.remove(&uuid) {
            let (canonical, aliases) = split_names(names);
            let category = category_rows.get(&uuid).copied().flatten();
            results.push(TagResponse { uuid, category, name: canonical, aliases });
        }
    }
    Ok(results)
}

pub async fn lookup_tag_by_id(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<TagResponse>, sqlx::Error> {
    let Some(uuid) = Uuid::parse_str(id).ok() else {
        return Ok(None);
    };

    let names = sqlx::query_as::<_, (String, i32)>(
        r#"
        SELECT name, role
        FROM tag_names
        WHERE tag_uuid = ?
        ORDER BY role
        "#,
    )
    .bind(uuid)
    .fetch_all(pool)
    .await?;

    if names.is_empty() {
        return Ok(None);
    }

    let (category,) = sqlx::query_as::<_, (Option<Uuid>,)>(
        r#"SELECT category FROM tags WHERE uuid = ?"#,
    )
    .bind(uuid)
    .fetch_one(pool)
    .await?;

    let (canonical, aliases) = split_names(names);
    Ok(Some(TagResponse {
        uuid,
        category,
        name: canonical,
        aliases,
    }))
}

pub async fn add_tag(pool: &SqlitePool, uuid: &Uuid, name: &str, aliases: &[String], category: Option<Uuid>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO tags (uuid, category)
        VALUES (?, ?)
        ON CONFLICT(uuid) DO UPDATE SET category=excluded.category
        "#,
    )
    .bind(uuid)
    .bind(category)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO tag_names (tag_uuid, name, role)
        VALUES (?, ?, 0)
        ON CONFLICT(tag_uuid, name) DO UPDATE SET role=0
        "#,
    )
    .bind(uuid)
    .bind(name.trim())
    .execute(pool)
    .await?;

    if !aliases.is_empty() {
        let values = aliases.iter().map(|_| "(?, ?, 1)").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "INSERT INTO tag_names (tag_uuid, name, role) VALUES {} ON CONFLICT(tag_uuid, name) DO UPDATE SET role=1",
            values
        );
        let mut query = sqlx::query(&sql);
        for alias in aliases {
            query = query.bind(uuid).bind(alias.trim());
        }
        query.execute(pool).await?;
    }
    Ok(())
}