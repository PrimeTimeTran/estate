pub struct DbProblemRepository {
	pool: sqlx::PgPool,
}
#[async_trait]
impl ProblemRepository for DbProblemRepository {
	async fn list(&self, request: &ListProblemsRequest) -> Result<ListProblemsResponse> {
		let problems = sqlx::query_as!(
			Problem,
			r#"
            SELECT ...
            FROM problems
            ORDER BY id
            LIMIT $1 OFFSET $2
            "#,
			request.page_size,
			request.page * request.page_size,
		)
		.fetch_all(&self.pool)
		.await?;

		// ...
		todo!()
	}

	// create/update/delete/get...
}
