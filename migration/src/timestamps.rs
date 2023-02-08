use sea_orm_migration::prelude::*;

pub fn timestamps(table: &mut TableCreateStatement) -> &mut TableCreateStatement {
    table
        .col(
            ColumnDef::new(Columns::CreatedAt)
                .timestamp_with_time_zone()
                .default(Expr::current_timestamp())
                .not_null(),
        )
        .col(
            ColumnDef::new(Columns::UpdatedAt)
                .timestamp_with_time_zone()
                .default(Expr::current_timestamp())
                .not_null(),
        )
}

#[derive(Iden)]
enum Columns {
    CreatedAt,
    UpdatedAt,
}
