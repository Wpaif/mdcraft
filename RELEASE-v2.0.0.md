## Mdcraft

**Mdcraft** é um aplicativo desktop para gerenciamento e análise de receitas de craft destinado a jogadores de **PXG**.

Ele permite calcular rapidamente **custos, receitas, lucros e margens**, combinando preços inseridos pelo usuário com dados obtidos automaticamente da **PxG Wiki**.  
Com persistência local e uma interface eficiente, o objetivo do Mdcraft é ajudar jogadores a **analisar e otimizar seus services**.

---

## v2.0.0 — 2026-03-15

Release principal com grandes melhorias de UX, feedback visual e busca.

---

## Changed

- Refatoração completa de popups, toasts e modais para lógica DRY e visual unificado.
- Animações e feedback visual aprimorados para todas as interações.
- Merge das branches de busca (elasticsearch) e feedback UI.
- Eliminação de todos os warnings do Clippy e limpeza de código.
- Ajustes finais em testes e dados para robustez.

## Added

- Barra de busca integrada com elasticsearch para receitas e itens.
- Sistema de feedback visual (toast) para ações do usuário.
- Novos atalhos e melhorias de navegação.
- Melhorias de performance e responsividade.

## Technical Details

- Refatoração de UI usando egui/eframe.
- Novos componentes DRY para popups/toasts.
- Testes automatizados revisados e expandidos.
- Scripts de build e sincronização de dados revisados.

## Known Limitations

- Sincronização de preços ainda depende da disponibilidade da PxG Wiki.
- Algumas regras de preço NPC permanecem estáticas.
- Limites de importação/exportação mantidos (1000 receitas, 2MB por JSON).

---

## Feedback

Encontrou um problema ou tem uma sugestão?
Abra uma **issue no repositório** para ajudar a melhorar o **Mdcraft**.

> Mdcraft é um projeto comunitário e não possui afiliação oficial com o **PokeXGames**.