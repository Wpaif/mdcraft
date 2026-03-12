#[derive(Clone, Debug)]
pub struct Item {
    pub nome: String,
    pub quantidade: u64,
    pub preco_unitario: u64,
    pub valor_total: u64,
    pub is_resource: bool,
    pub preco_input: String,
}

