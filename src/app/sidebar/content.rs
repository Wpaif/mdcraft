use eframe::egui;

use crate::app::{MdcraftApp, SavedCraft, apply_saved_item_prices, capture_saved_item_prices};
use crate::parse::parse_clipboard;

use super::{capitalize_display_name, json_io, placeholder};

fn apply_pending_sidebar_actions(
    app: &mut MdcraftApp,
    pending_click_delete: Option<usize>,
    pending_click_select: Option<usize>,
) {
    if let Some(idx) = pending_click_delete {
        app.pending_delete_index = Some(idx);
    }

    if let Some(idx) = pending_click_select {
        load_saved_craft_for_edit(app, idx);
    }
}

fn toggle_sidebar(app: &mut MdcraftApp, clicked: bool) {
    if clicked {
        app.sidebar_open = !app.sidebar_open;
    }
}

fn set_pending_action(slot: &mut Option<usize>, idx: usize, clicked: bool) {
    if clicked {
        *slot = Some(idx);
    }
}

fn start_save_recipe_prompt(app: &mut MdcraftApp, save_clicked: bool) {
    if save_clicked {
        app.awaiting_craft_name = true;
        app.pending_craft_name = infer_craft_name_from_items(app).unwrap_or_default();
        app.focus_craft_name_input = true;
    }
}

fn infer_craft_name_from_items(app: &MdcraftApp) -> Option<String> {
    crate::app::infer_craft_name_from_items(
        &app.items,
        &app.craft_recipes_cache,
        &app.craft_recipe_name_by_signature,
    )
    .map(|name| capitalize_display_name(&name))
}

fn sidebar_header_bg_color(
    hovered: bool,
    hovered_bg: egui::Color32,
    inactive_bg: egui::Color32,
) -> egui::Color32 {
    if hovered { hovered_bg } else { inactive_bg }
}

pub(super) fn render_sidebar_content(ui: &mut egui::Ui, app: &mut MdcraftApp, content_w: f32) {
    let content_w = content_w.max(120.0);
    let has_saved_crafts = !app.saved_crafts.is_empty();

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(10.0);

    egui::TopBottomPanel::bottom(egui::Id::new("sidebar_json_actions_bottom"))
        .show_separator_line(false)
        .resizable(false)
        .show_inside(ui, |ui| {
            json_io::render_sidebar_json_actions(ui, app, content_w, has_saved_crafts);
        });

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(ui.available_height().max(120.0))
        .show(ui, |ui| {
            let has_recipe = !app.input_text.trim().is_empty() && !app.items.is_empty();
            if has_recipe {
                let save_button = egui::Button::new(
                    egui::RichText::new("Salvar receita atual")
                        .strong()
                        .color(egui::Color32::from_rgb(245, 251, 244)),
                )
                .fill(egui::Color32::from_rgb(48, 118, 78))
                .stroke(egui::Stroke::new(
                    1.0,
                    egui::Color32::from_rgb(86, 168, 120),
                ))
                .corner_radius(egui::CornerRadius::same(8));

                let save_clicked = ui
                    .add_sized([content_w, 34.0], save_button)
                    .on_hover_text("Salvar a receita atual com nome automático ou manual")
                    .clicked();
                start_save_recipe_prompt(app, save_clicked);
            } else {
                ui.label(egui::RichText::new("Adicione uma receita para salvar").weak());
            }

            if app.awaiting_craft_name {
                ui.add_space(10.0);
                ui.label(egui::RichText::new("Nome da receita:").strong());

                let mut name_resp_opt: Option<egui::Response> = None;
                let accent = ui.visuals().hyperlink_color;
                egui::Frame::NONE
                    .fill(egui::Color32::from_rgba_unmultiplied(
                        accent.r(),
                        accent.g(),
                        accent.b(),
                        18,
                    ))
                    .stroke(egui::Stroke::new(
                        1.0,
                        egui::Color32::from_rgba_unmultiplied(
                            accent.r(),
                            accent.g(),
                            accent.b(),
                            96,
                        ),
                    ))
                    .corner_radius(egui::CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(6, 4))
                    .show(ui, |ui| {
                        let input_width = (content_w - 12.0).max(80.0);
                        let name_resp = ui.add_sized(
                            [input_width, 30.0],
                            egui::TextEdit::singleline(&mut app.pending_craft_name)
                                .font(egui::TextStyle::Button)
                                .horizontal_align(egui::Align::Center)
                                .vertical_align(egui::Align::Center)
                                .hint_text(placeholder(ui, "Digite um nome ou pressione Enter")),
                        );
                        name_resp_opt = Some(name_resp);
                    });

                let name_resp = name_resp_opt.expect("name input response should exist");

                if app.focus_craft_name_input {
                    name_resp.request_focus();
                    app.focus_craft_name_input = false;
                }

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    app.awaiting_craft_name = false;
                    app.pending_craft_name.clear();
                    app.focus_craft_name_input = false;
                }

                let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let save_by_enter = enter_pressed;

                if save_by_enter {
                    let fallback_name = format!("Receita {}", app.saved_crafts.len() + 1);
                    let raw_name = if app.pending_craft_name.trim().is_empty() {
                        fallback_name
                    } else {
                        app.pending_craft_name.clone()
                    };
                    let normalized_name = capitalize_display_name(&raw_name);
                    app.saved_crafts.insert(
                        0,
                        SavedCraft {
                            name: normalized_name,
                            recipe_text: app.input_text.clone(),
                            sell_price_input: app.sell_price_input.clone(),
                            item_prices: capture_saved_item_prices(&app.items),
                        },
                    );
                    app.active_saved_craft_index = app.active_saved_craft_index.map(|idx| idx + 1);
                    app.persist_saved_crafts_to_sqlite();
                    app.awaiting_craft_name = false;
                    app.pending_craft_name.clear();
                    app.focus_craft_name_input = false;
                }
            }

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Receitas salvas").strong());
            ui.add_space(6.0);

            if app.saved_crafts.is_empty() {
                ui.label(egui::RichText::new("Nenhuma receita salva ainda.").weak());
            } else {
                let mut pending_click_delete: Option<usize> = None;
                let mut pending_click_select: Option<usize> = None;

                for (idx, craft) in app.saved_crafts.iter().enumerate() {
                    let is_active = app.active_saved_craft_index == Some(idx);
                    let name_text = capitalize_display_name(&craft.name);
                    let saved_lines = craft
                        .recipe_text
                        .lines()
                        .filter(|line| !line.trim().is_empty())
                        .count();
                    let has_saved_price = !craft.sell_price_input.trim().is_empty();
                    let hover_details = if has_saved_price {
                        format!("{} linhas salvas | com preco final", saved_lines)
                    } else {
                        format!("{} linhas salvas", saved_lines)
                    };

                    let item_fill = if is_active {
                        ui.visuals().faint_bg_color
                    } else {
                        ui.visuals().widgets.inactive.bg_fill
                    };
                    let item_stroke = if is_active {
                        ui.visuals().widgets.active.bg_stroke
                    } else {
                        ui.visuals().widgets.inactive.bg_stroke
                    };

                    egui::Frame::new()
                        .fill(item_fill)
                        .stroke(item_stroke)
                        .corner_radius(egui::CornerRadius::same(4))
                        .inner_margin(egui::Margin::symmetric(8, 5))
                        .show(ui, |ui| {
                            ui.set_width(content_w);
                            let row_height = 22.0;
                            let icon_size = 20.0;
                            ui.allocate_ui_with_layout(
                                egui::vec2(content_w, row_height),
                                egui::Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let text_width =
                                        (content_w - icon_size - ui.spacing().item_spacing.x - 8.0)
                                            .max(80.0);

                                    let name_btn = egui::Button::new(
                                        egui::RichText::new(&name_text)
                                            .size(14.0)
                                            .color(ui.visuals().text_color()),
                                    )
                                    .fill(egui::Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::NONE);

                                    let name_resp = ui
                                        .add_sized([text_width, icon_size], name_btn)
                                        .on_hover_text(hover_details);
                                    set_pending_action(
                                        &mut pending_click_select,
                                        idx,
                                        name_resp.clicked(),
                                    );

                                    let (delete_rect, delete_resp) = ui.allocate_exact_size(
                                        egui::vec2(icon_size, icon_size),
                                        egui::Sense::click(),
                                    );
                                    let delete_fill = if delete_resp.hovered() {
                                        egui::Color32::from_rgba_unmultiplied(220, 98, 98, 44)
                                    } else {
                                        egui::Color32::from_rgba_unmultiplied(220, 98, 98, 32)
                                    };
                                    ui.painter().rect(
                                        delete_rect,
                                        egui::CornerRadius::same(4),
                                        delete_fill,
                                        egui::Stroke::new(
                                            1.0,
                                            egui::Color32::from_rgb(180, 72, 72),
                                        ),
                                        egui::StrokeKind::Middle,
                                    );
                                    ui.painter().text(
                                        delete_rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "🗑",
                                        egui::FontId::proportional(13.0),
                                        egui::Color32::from_rgb(220, 98, 98),
                                    );

                                    let delete_clicked =
                                        delete_resp.on_hover_text("Excluir receita").clicked();
                                    set_pending_action(
                                        &mut pending_click_delete,
                                        idx,
                                        delete_clicked,
                                    );
                                },
                            );
                        });
                    ui.add_space(4.0);
                }

                apply_pending_sidebar_actions(app, pending_click_delete, pending_click_select);
            }
        });
}

fn load_saved_craft_for_edit(app: &mut MdcraftApp, idx: usize) {
    let Some(craft) = app.saved_crafts.get(idx) else {
        return;
    };

    app.input_text = craft.recipe_text.clone();
    app.sell_price_input = craft.sell_price_input.clone();

    let resources: Vec<&str> = app.resource_list.iter().map(AsRef::as_ref).collect();
    app.items = parse_clipboard(&app.input_text, &resources);
    apply_saved_item_prices(&mut app.items, &craft.item_prices);
    app.active_saved_craft_index = Some(idx);
}

pub(super) fn render_sidebar_header(ui: &mut egui::Ui, app: &mut MdcraftApp) {
    ui.horizontal(|ui| {
        let toggle_icon = if app.sidebar_open { "◀" } else { "▶" };
        let (rect, resp) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::click());

        let bg = sidebar_header_bg_color(
            resp.hovered(),
            ui.visuals().widgets.hovered.bg_fill,
            ui.visuals().widgets.inactive.bg_fill,
        );
        ui.painter()
            .rect_filled(rect, egui::CornerRadius::same(6), bg);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            toggle_icon,
            egui::TextStyle::Button.resolve(ui.style()),
            ui.visuals().text_color(),
        );

        toggle_sidebar(app, resp.clicked());

        if app.sidebar_open {
            ui.label(egui::RichText::new("RECEITAS").strong());
        }
    });
}

#[cfg(test)]
mod tests {
    use eframe::egui;

    use crate::app::{MdcraftApp, SavedCraft, SavedItemPrice};
    use crate::data::wiki_scraper::{
        CraftIngredient, CraftProfession, CraftRank, ScrapedCraftRecipe,
    };

    use super::{
        apply_pending_sidebar_actions, infer_craft_name_from_items, load_saved_craft_for_edit,
        render_sidebar_content, render_sidebar_header, set_pending_action, sidebar_header_bg_color,
        start_save_recipe_prompt, toggle_sidebar,
    };

    fn run_sidebar_content_with_events(
        app: &mut MdcraftApp,
        events: Vec<egui::Event>,
        content_w: f32,
    ) {
        let ctx = egui::Context::default();
        let mut input = egui::RawInput::default();
        input.events = events;

        let _ = ctx.run(input, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_sidebar_content(ui, app, content_w);
            });
        });
    }

    fn make_saved_craft(name: &str, recipe_text: &str, sell_price_input: &str) -> SavedCraft {
        SavedCraft {
            name: name.to_string(),
            recipe_text: recipe_text.to_string(),
            sell_price_input: sell_price_input.to_string(),
            item_prices: vec![],
        }
    }

    #[test]
    fn load_saved_craft_for_edit_updates_active_data() {
        let mut app = MdcraftApp::default();
        app.saved_crafts.push(SavedCraft {
            name: "teste".to_string(),
            recipe_text: "2 Iron Ore, 3 Screw".to_string(),
            sell_price_input: "12k".to_string(),
            item_prices: vec![SavedItemPrice {
                item_name: "Screw".to_string(),
                price_input: "250".to_string(),
            }],
        });

        load_saved_craft_for_edit(&mut app, 0);

        assert_eq!(app.active_saved_craft_index, Some(0));
        assert_eq!(app.sell_price_input, "12k");
        assert!(!app.items.is_empty());
        let screw = app
            .items
            .iter()
            .find(|i| i.nome == "Screw")
            .expect("Screw should exist after loading saved craft");
        assert_eq!(screw.preco_input, "250");
        assert_eq!(screw.preco_unitario, 250.0);
    }

    #[test]
    fn load_saved_craft_for_edit_ignores_out_of_bounds_index() {
        let mut app = MdcraftApp::default();
        app.input_text = "unchanged".to_string();
        app.saved_crafts
            .push(make_saved_craft("teste", "1 Iron Ore", "1k"));

        load_saved_craft_for_edit(&mut app, 9);

        assert_eq!(app.input_text, "unchanged");
        assert_eq!(app.active_saved_craft_index, None);
    }

    #[test]
    fn render_sidebar_header_and_content_do_not_panic() {
        let mut app = MdcraftApp::default();
        app.input_text = "1 Iron Ore".to_string();
        app.items = vec![];

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_sidebar_header(ui, &mut app);
                render_sidebar_content(ui, &mut app, 220.0);
            });
        });
    }

    #[test]
    fn render_sidebar_content_handles_saved_recipes_list() {
        let mut app = MdcraftApp::default();
        app.sidebar_open = true;
        app.saved_crafts
            .push(make_saved_craft("receita a", "1 Iron Ore", "3k"));
        app.saved_crafts
            .push(make_saved_craft("receita b", "2 Screw", ""));
        app.active_saved_craft_index = Some(0);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                render_sidebar_content(ui, &mut app, 260.0);
            });
        });

        assert_eq!(app.saved_crafts.len(), 2);
    }

    #[test]
    fn sidebar_header_bg_color_selects_hover_or_inactive_color() {
        let hovered = sidebar_header_bg_color(
            true,
            egui::Color32::from_rgb(10, 20, 30),
            egui::Color32::from_rgb(40, 50, 60),
        );
        assert_eq!(hovered, egui::Color32::from_rgb(10, 20, 30));

        let inactive = sidebar_header_bg_color(
            false,
            egui::Color32::from_rgb(10, 20, 30),
            egui::Color32::from_rgb(40, 50, 60),
        );
        assert_eq!(inactive, egui::Color32::from_rgb(40, 50, 60));
    }

    #[test]
    fn render_sidebar_content_handles_pending_name_input_state() {
        let mut app = MdcraftApp::default();
        app.sidebar_open = true;
        app.awaiting_craft_name = true;
        app.focus_craft_name_input = true;
        app.pending_craft_name = "Minha Receita".to_string();
        app.input_text = "1 Iron Ore".to_string();
        app.items = vec![crate::model::Item {
            nome: "Iron Ore".to_string(),
            quantidade: 1,
            preco_unitario: 0.0,
            valor_total: 0.0,
            is_resource: true,
            preco_input: String::new(),
        }];

        run_sidebar_content_with_events(&mut app, vec![], 260.0);

        assert!(app.awaiting_craft_name);
    }

    #[test]
    fn render_sidebar_content_escape_cancels_name_prompt() {
        let mut app = MdcraftApp::default();
        app.awaiting_craft_name = true;
        app.pending_craft_name = "Tmp".to_string();
        app.focus_craft_name_input = true;

        run_sidebar_content_with_events(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Escape,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            260.0,
        );

        assert!(!app.awaiting_craft_name);
        assert!(app.pending_craft_name.is_empty());
        assert!(!app.focus_craft_name_input);
    }

    #[test]
    fn render_sidebar_content_enter_saves_pending_recipe_name() {
        let mut app = MdcraftApp::default();
        app.awaiting_craft_name = true;
        app.pending_craft_name = "nova receita".to_string();
        app.input_text = "1 Iron Ore".to_string();
        app.sell_price_input = "9k".to_string();
        app.active_saved_craft_index = Some(1);

        run_sidebar_content_with_events(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            260.0,
        );

        assert!(!app.awaiting_craft_name);
        assert_eq!(app.saved_crafts.len(), 1);
        assert_eq!(app.saved_crafts[0].name, "Nova Receita");
        assert_eq!(app.saved_crafts[0].sell_price_input, "9k");
        assert_eq!(app.active_saved_craft_index, Some(2));
    }

    #[test]
    fn render_sidebar_content_enter_uses_fallback_name_when_blank() {
        let mut app = MdcraftApp::default();
        app.awaiting_craft_name = true;
        app.pending_craft_name = "   ".to_string();
        app.input_text = "1 Iron Ore".to_string();

        run_sidebar_content_with_events(
            &mut app,
            vec![egui::Event::Key {
                key: egui::Key::Enter,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            }],
            260.0,
        );

        assert_eq!(app.saved_crafts[0].name, "Receita 1");
    }

    #[test]
    fn apply_pending_sidebar_actions_sets_delete_and_selects_recipe() {
        let mut app = MdcraftApp::default();
        app.saved_crafts
            .push(make_saved_craft("receita a", "1 Iron Ore", "3k"));

        apply_pending_sidebar_actions(&mut app, Some(0), Some(0));

        assert_eq!(app.pending_delete_index, Some(0));
        assert_eq!(app.active_saved_craft_index, Some(0));
    }

    #[test]
    fn toggle_sidebar_flips_only_when_clicked() {
        let mut app = MdcraftApp::default();
        app.sidebar_open = true;

        toggle_sidebar(&mut app, false);
        assert!(app.sidebar_open);

        toggle_sidebar(&mut app, true);
        assert!(!app.sidebar_open);
    }

    #[test]
    fn set_pending_action_respects_clicked_flag() {
        let mut slot = None;

        set_pending_action(&mut slot, 1, false);
        assert_eq!(slot, None);

        set_pending_action(&mut slot, 3, true);
        assert_eq!(slot, Some(3));
    }

    #[test]
    fn start_save_recipe_prompt_only_changes_state_on_click() {
        let mut app = MdcraftApp::default();
        app.pending_craft_name = "keep".to_string();

        start_save_recipe_prompt(&mut app, false);
        assert!(!app.awaiting_craft_name);
        assert_eq!(app.pending_craft_name, "keep");

        start_save_recipe_prompt(&mut app, true);
        assert!(app.awaiting_craft_name);
        assert_eq!(app.pending_craft_name, "");
        assert!(app.focus_craft_name_input);
    }

    #[test]
    fn infer_craft_name_from_items_returns_exact_match() {
        let mut app = MdcraftApp::default();
        app.items = vec![
            crate::model::Item {
                nome: "Apricorn".to_string(),
                quantidade: 1,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: false,
                preco_input: String::new(),
            },
            crate::model::Item {
                nome: "Screw".to_string(),
                quantidade: 80,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: false,
                preco_input: String::new(),
            },
        ];

        app.craft_recipes_cache = vec![ScrapedCraftRecipe {
            profession: CraftProfession::Engineer,
            rank: CraftRank::E,
            name: "poke ball (100x)".to_string(),
            ingredients: vec![
                CraftIngredient {
                    name: "Apricorn".to_string(),
                    quantity: 1.0,
                },
                CraftIngredient {
                    name: "Screw".to_string(),
                    quantity: 80.0,
                },
            ],
        }];
        app.rebuild_craft_recipe_name_index();

        let inferred = infer_craft_name_from_items(&app);
        assert_eq!(inferred.as_deref(), Some("Poke Ball (100x)"));
    }

    #[test]
    fn infer_craft_name_from_items_accepts_multiple_recipe_units() {
        let mut app = MdcraftApp::default();
        app.items = vec![
            crate::model::Item {
                nome: "Tech Data".to_string(),
                quantidade: 720,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: false,
                preco_input: String::new(),
            },
            crate::model::Item {
                nome: "Wolf Tail".to_string(),
                quantidade: 24,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: false,
                preco_input: String::new(),
            },
            crate::model::Item {
                nome: "Black Wool Ball".to_string(),
                quantidade: 12,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: false,
                preco_input: String::new(),
            },
            crate::model::Item {
                nome: "Compressed Nightmare Gem".to_string(),
                quantidade: 2,
                preco_unitario: 0.0,
                valor_total: 0.0,
                is_resource: false,
                preco_input: String::new(),
            },
        ];

        app.craft_recipes_cache = vec![ScrapedCraftRecipe {
            profession: CraftProfession::Engineer,
            rank: CraftRank::S,
            name: "Enhancement Kit".to_string(),
            ingredients: vec![
                CraftIngredient {
                    name: "Tech Data".to_string(),
                    quantity: 360.0,
                },
                CraftIngredient {
                    name: "Wolf Tail".to_string(),
                    quantity: 12.0,
                },
                CraftIngredient {
                    name: "Black Wool Ball".to_string(),
                    quantity: 6.0,
                },
                CraftIngredient {
                    name: "Compressed Nightmare Gem".to_string(),
                    quantity: 1.0,
                },
            ],
        }];
        app.rebuild_craft_recipe_name_index();

        let inferred = infer_craft_name_from_items(&app);
        assert_eq!(inferred.as_deref(), Some("Enhancement Kit"));
    }

    #[test]
    fn start_save_recipe_prompt_prefills_inferred_craft_name() {
        let mut app = MdcraftApp::default();
        app.items = vec![crate::model::Item {
            nome: "Diamond".to_string(),
            quantidade: 1,
            preco_unitario: 0.0,
            valor_total: 0.0,
            is_resource: false,
            preco_input: String::new(),
        }];
        app.craft_recipes_cache = vec![ScrapedCraftRecipe {
            profession: CraftProfession::Stylist,
            rank: CraftRank::E,
            name: "Diamond Dust (20x)".to_string(),
            ingredients: vec![CraftIngredient {
                name: "Diamond".to_string(),
                quantity: 1.0,
            }],
        }];
        app.rebuild_craft_recipe_name_index();

        start_save_recipe_prompt(&mut app, true);

        assert!(app.awaiting_craft_name);
        assert_eq!(app.pending_craft_name, "Diamond Dust (20x)");
        assert!(app.focus_craft_name_input);
    }
}
