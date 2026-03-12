use crate::model::Item;

pub fn parse_price_flag(valor: &str) -> Result<u64, String> {
    let valor = valor.trim().to_lowercase();

    if valor.is_empty() {
        return Ok(0);
    }

    if valor.ends_with("kk") {
        let numero = valor
            .trim_end_matches("kk")
            .parse::<f64>()
            .map_err(|_| "valor inválido")?;

        Ok((numero * 1_000_000.0) as u64)
    } else if valor.ends_with('k') {
        let numero = valor
            .trim_end_matches('k')
            .parse::<f64>()
            .map_err(|_| "valor inválido")?;

        Ok((numero * 1_000.0) as u64)
    } else {
        valor
            .parse::<f64>()
            .map(|f| f as u64)
            .map_err(|_| "valor inválido".to_string())
    }
}

pub fn parse_clipboard(clipboard_content: &str, resource_list: &[&str]) -> Vec<Item> {
    if clipboard_content.trim().is_empty() {
        return Vec::new();
    }

    let mut items = Vec::new();

    for item_str in clipboard_content.split(',') {
        let item_str = item_str.trim();

        if item_str.is_empty() {
            continue;
        }

        let parts: Vec<&str> = item_str.splitn(2, ' ').collect();

        if parts.len() == 2 {
            if let Ok(quantidade) = parts[0].parse::<u64>() {
                // Remove pontos finais e espaços que podem vir do copy-paste do jogo
                let nome_original = parts[1].trim().trim_end_matches('.');
                let nome_lower = nome_original.to_lowercase();

                // Validação de plural / singular contra a lista de resources base
                let mut is_resource = false;
                for res in resource_list {
                    let res_lower = res.to_lowercase();

                    if nome_lower == res_lower
                        || nome_lower == format!("{}s", res_lower)
                        || nome_lower == format!("{}es", res_lower)
                        || res_lower == format!("{}s", nome_lower)
                        || res_lower == format!("{}es", nome_lower)
                    {
                        is_resource = true;
                        break;
                    }
                }

                items.push(Item {
                    nome: nome_original.to_string(),
                    quantidade,
                    preco_unitario: 0,
                    valor_total: 0,
                    is_resource,
                    preco_input: String::new(),
                });
            }
        }
    }

    items
}
