use crate::models::SearchFilter;

/// Build a parameterized WHERE clause from a SearchFilter
/// Returns the WHERE clause string and a vector of parameters
pub fn build_where_clause(filter: &SearchFilter) -> (String, Vec<Box<dyn rusqlite::ToSql>>) {
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref date_from) = filter.date_from {
        clauses.push(format!("AND date_taken >= ?{}", params.len() + 3));
        params.push(Box::new(date_from.clone()));
    }

    if let Some(ref date_to) = filter.date_to {
        clauses.push(format!("AND date_taken <= ?{}", params.len() + 3));
        params.push(Box::new(date_to.clone()));
    }

    if let Some(ref media_type) = filter.media_type {
        clauses.push(format!("AND media_type = ?{}", params.len() + 3));
        params.push(Box::new(media_type.clone()));
    }

    if let Some(ref camera_model) = filter.camera_model {
        clauses.push(format!("AND camera_model = ?{}", params.len() + 3));
        params.push(Box::new(camera_model.clone()));
    }

    if let Some(ref extension) = filter.extension {
        clauses.push(format!("AND extension = ?{}", params.len() + 3));
        params.push(Box::new(extension.clone()));
    }

    if let Some(ref query) = filter.query {
        let like_pattern = format!("%{}%", query);
        let param_start = params.len() + 3;
        clauses.push(format!(
            "AND (camera_model LIKE ?{p} OR camera_make LIKE ?{p} OR lens_model LIKE ?{p} OR extension LIKE ?{p})",
            p = param_start
        ));
        params.push(Box::new(like_pattern));
    }

    (clauses.join(" "), params)
}
