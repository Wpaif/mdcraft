#[derive(Clone, Debug)]
pub struct Item {
    pub nome: String,
    pub quantidade: u64,
    pub quantidade_base: u64,
    pub preco_unitario: f64,
    pub valor_total: f64,
    pub is_resource: bool,
    pub preco_input: String,
}
