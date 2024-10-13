use sea_orm::DbErr;

#[derive(Debug)]
pub enum ModelError {
    DBError(DbErr),
    Empty,
}