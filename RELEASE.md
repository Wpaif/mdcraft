## Mdcraft

**Mdcraft** é um aplicativo desktop para gerenciamento e análise de receitas de craft.  
Ele permite calcular **custos, receitas e margens de lucro** rapidamente, com integração automática de preços via wiki, persistência local e uma interface eficiente para otimizar produções.

---

## v1.0.0 — 2026-03-14

Primeira versão estável do **Mdcraft**.

Esta release introduz o núcleo da aplicação, incluindo análise de receitas, sincronização automática de preços e persistência local de dados.

---

## Added

Novas funcionalidades incluídas nesta versão:

- Aplicativo desktop multiplataforma (**Linux** e **Windows**).
- Cálculo automático de **custo, receita, lucro e margem** de crafts.
- Conversão automática de **receitas em texto para grade de crafting**.
- Comparação entre **preços inseridos pelo usuário** e **preços de NPC**.
- **Regras fixas de preço NPC** para itens específicos.
- **Sincronização automática de preços via wiki**, executada diariamente após **07:40**.
- Persistência local de:
  - configurações
  - receitas
  - preços por item
- Armazenamento de dados utilizando **SQLite**.
- **Importação e exportação de receitas em JSON**.
- **Atalhos de teclado** para navegação rápida:
  - `Ctrl + E`
  - `Esc`
  - `Enter`

---

## Technical Details

Informações técnicas relevantes desta release:

- Sincronização de preços executada em **background**.
- Pipeline paralela para consulta de **múltiplas fontes de preço**.
- Sistema de **tema claro/escuro**, com:
  - detecção automática do sistema
  - alternância manual pelo usuário.
- Interface construída utilizando **egui / eframe**.
- Scripts e instruções para **build multiplataforma**:
  - Linux
  - Windows (MSVC)
  - Windows (GNU)
- **Testes automatizados** executados via `cargo test`.

---

## Known Limitations

Limitações conhecidas desta versão:

- A sincronização de preços depende da **disponibilidade da wiki**.
- Algumas regras de preço NPC são **estáticas** e podem exigir atualização manual.
- Importação e exportação limitadas a:
  - **1000 receitas**
  - **2MB por arquivo JSON**

---

## Feedback

Encontrou um problema ou tem uma sugestão?

Abra uma **issue no repositório** para ajudar a melhorar o **Mdcraft**.