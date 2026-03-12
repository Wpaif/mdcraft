use std::env;
use std::io::{self, Write};

struct Item {
    nome: String,
    quantidade: u64,
    preco_unitario: u64,
    valor_total: u64,
    is_resource: bool,
}

fn format_game_units(valor: f64) -> String {
    if valor >= 1_000_000.0 {
        format!("{:.1}KK", valor / 1_000_000.0)
    } else if valor >= 1_000.0 {
        let k = valor / 1_000.0;
        if (k * 10.0) % 10.0 == 0.0 {
            format!("{}k", k as u64)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        format!("{:.0}", valor)
    }
}

fn parse_price_flag(valor: &str) -> Result<u64, String> {
    let valor = valor.trim().to_lowercase();
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

fn parse_clipboard(input: &str, resources: &[&str]) -> Vec<Item> {
    input
        .split(',')
        .filter_map(|parte| {
            let parte = parte.trim().trim_end_matches('.');
            let mut split = parte.splitn(2, ' ');
            let quantidade = split.next()?.parse().ok()?;
            let mut nome = split.next()?.trim().to_string();
            let is_resource = resources
                .iter()
                .any(|r| nome.to_lowercase().contains(&r.to_lowercase()));
            if is_resource {
                nome = nome.replace("(resource)", "").trim().to_string();
            }
            Some(Item {
                nome,
                quantidade,
                preco_unitario: 0,
                valor_total: 0,
                is_resource,
            })
        })
        .collect()
}

fn ler_precos(itens: &mut Vec<Item>) {
    for item in itens.iter_mut() {
        if item.is_resource {
            item.preco_unitario = 0;
            item.valor_total = 0;
            continue;
        }
        let preco_unitario = loop {
            let mut input = String::new();
            print!("{} price> ", item.nome);
            io::stdout().flush().unwrap();
            io::stdin().read_line(&mut input).unwrap();
            match parse_price_flag(&input) {
                Ok(v) => break v,
                Err(_) => {
                    println!("Formato inválido. Use unidades, k ou kk, ex: 1000, 1k, 1kk, 0.1kk.")
                }
            }
        };
        item.preco_unitario = preco_unitario;
        item.valor_total = preco_unitario * item.quantidade;
    }
}

fn main() {
    let resource_list = [
        "tech data",
        "iron ore",
        "iron bar",
        "platinum bar",
        "platinum ore",
    ];

    let args: Vec<String> = env::args().collect();
    let mut sell_price: Option<u64> = None;
    if let Some(pos) = args.iter().position(|x| x == "--price") {
        if args.len() > pos + 1 {
            match parse_price_flag(&args[pos + 1]) {
                Ok(v) => sell_price = Some(v),
                Err(_) => {
                    eprintln!("Use --price com número seguido de k ou kk");
                    return;
                }
            }
        } else {
            eprintln!("Use --price <valor>");
            return;
        }
    }

    println!("Cole a lista do craft do jogo:");
    let mut entrada = String::new();
    io::stdin().read_line(&mut entrada).unwrap();
    let mut itens = parse_clipboard(&entrada, &resource_list);

    if itens.is_empty() {
        println!("Nenhum item detectado.");
        return;
    }

    ler_precos(&mut itens);

    if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(&["/C", "cls"])
            .status()
            .unwrap();
    } else {
        std::process::Command::new("clear").status().unwrap();
    }

    println!("\nResultados:");
    let mut total: u64 = 0;
    let mut resource_points = 0;
    let mut resource_name = String::new();

    for item in &itens {
        if !item.is_resource {
            println!(
                "{:<25} {:>5} x {:>10} = {}",
                item.nome,
                item.quantidade,
                format_game_units(item.preco_unitario as f64),
                format_game_units(item.valor_total as f64)
            );
            total += item.valor_total;
        }
    }

    for item in &itens {
        if item.is_resource {
            resource_points = item.quantidade;
            resource_name = item.nome.clone();
            println!("{:<25} {:>5} (resource)", item.nome, item.quantidade);
        }
    }

    println!("--------------------------------");
    println!("CUSTO TOTAL: {}", format_game_units(total as f64));

    if let Some(venda) = sell_price {
        let lucro_total = venda.saturating_sub(total);
        let margem = if total > 0 {
            lucro_total as f64 / total as f64 * 100.0
        } else {
            0.0
        };
        println!("PREÇO VENDA: {}", format_game_units(venda as f64));
        println!("LUCRO: {}", format_game_units(lucro_total as f64));
        println!("MARGEM: {:.1}%", margem);

        if resource_points > 0 {
            let custo_por_ponto = lucro_total as f64 / resource_points as f64;
            println!(
                "CUSTO POR PONTO ({} {}): {:.1}",
                resource_points, resource_name, custo_por_ponto
            );
        }
    }
}
