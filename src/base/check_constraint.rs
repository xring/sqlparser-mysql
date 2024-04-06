/// `[CONSTRAINT [symbol]] CHECK (expr) [[NOT] ENFORCED]`
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CheckConstraintDefinition {
    pub symbol: Option<String>,
    pub expr: String,
    pub enforced: bool,
}
