use eframe::egui::{CentralPanel, Context, RichText};

pub struct InitError {
    error: String,
    hyperlink: String,
}

impl InitError {
    pub fn new(error: tegra_rcm::SwitchError) -> Self {
        let error = gen_error(&error);
        Self {
            error: error.0,
            hyperlink: error.1,
        }
    }
}

impl eframe::App for InitError {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                    ui.vertical(|ui| {
                        ui.add_space(20.);
                        ui.label(RichText::new("An error occurred while initializing the app, please correct it and relaunch the app.").size(26.));
                        ui.add_space(40.);
                        ui.label(RichText::new(&self.error).size(20.));
                        ui.hyperlink_to(RichText::new(&self.hyperlink).size(20.), &self.hyperlink);
                    });
            });
        });
    }
}

pub(crate) fn gen_error(error: &tegra_rcm::SwitchError) -> (String, String) {
    match error {
        tegra_rcm::SwitchError::UdevRulesNotFound => {
            let error = "Udev rules not installed and must be installed separately, see the following for instructions";
            let link = "https://budde25.github.io/switcheroo/troubleshooting/#linux-permission-denied-error";
            (error.to_string(), link.to_string())
        }
        tegra_rcm::SwitchError::AccessDenied => {
            let link = "https://budde25.github.io/switcheroo/troubleshooting/#linux-permission-denied-error";
            let error = "USB permission error, see the following to troubleshoot:";
            (error.to_string(), link.to_string())
        }
        tegra_rcm::SwitchError::WindowsWrongDriver(i) => {
            let error = format!("Wrong USB driver installed, expected libusbK but found `{i}`, see the following to troubleshoot:");
            let link =
                "https://budde25.github.io/switcheroo/troubleshooting/#windows-wrong-driver-error";
            (error, link.to_string())
        }
        e => (
            e.to_string(),
            "https://budde25.github.io/switcheroo/troubleshooting/".to_string(),
        ),
    }
}
