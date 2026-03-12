use eframe::egui;

use super::styles::{setup_custom_styles, setup_emoji_support};
use super::theme_state::theme_toggle_button;
use super::ui_sections::{
    collect_found_resources, render_closing, render_craft_input, render_items_and_values,
};

impl eframe::App for super::MdcraftApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.fonts_loaded {
            setup_custom_styles(ctx);
            setup_emoji_support(ctx);
            ctx.set_visuals(self.theme.visuals());
            self.fonts_loaded = true;
        }

        egui::Area::new(egui::Id::new("theme_toggle_area"))
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                if theme_toggle_button(ui, self.theme)
                    .on_hover_text("Alternar tema")
                    .clicked()
                {
                    self.theme = self.theme.toggle();
                    ctx.set_visuals(self.theme.visuals());
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let available_width = ui.available_width();
            let max_width = available_width.min(1600.0);
            let padding = ((available_width - max_width) / 2.0).max(10.0) as i8;

            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(padding, 20))
                .show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.heading(egui::RichText::new("Mdcraft Calculator").strong());
                            });

                            ui.add_space(20.0);
                            let content_width = ui.available_width();

                            render_craft_input(ui, self, content_width);

                            ui.add_space(20.0);

                            let mut total_cost: u64 = 0;
                            let found_resources = collect_found_resources(self);

                            render_items_and_values(ui, self, content_width, &mut total_cost);

                            ui.add_space(20.0);

                            render_closing(ui, self, content_width, total_cost, &found_resources);
                        });
                });
        });
    }
}
