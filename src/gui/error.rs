use eframe::egui::{CentralPanel, Context, Direction, Layout};

pub struct InitError {
    error: String,
}

impl InitError {
    pub fn new(error: tegra_rcm::SwitchError) -> Self {
        Self {
            error: gen_error(&error).unwrap(),
        }
    }
}

impl eframe::App for InitError {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                    ui.heading(&self.error);
                    ui.label("Unrecoverable error, please correct this error and relaunch the app");
                })
            });
        });
    }
}

pub(crate) fn gen_error(error: &tegra_rcm::SwitchError) -> Option<String> {
    match error {
        tegra_rcm::SwitchError::AccessDenied => {
            let link = "https://budde25.github.io/switcheroo/troubleshooting/#linux-permission-denied-error";
            Some(format!(
                "USB permission error, see the following to troubleshoot\n{link}"
            ))
        }
        tegra_rcm::SwitchError::WindowsWrongDriver(i) => {
            let link =
                "https://budde25.github.io/switcheroo/troubleshooting/#windows-wrong-driver-error";
            Some(format!(
                "Wrong USB driver installed, expected libusbK but found `{i}`, see the following to troubleshoot\n{link}"
            ))
        }
        tegra_rcm::SwitchError::SwitchNotFound => None,
        e => Some(e.to_string()),
    }
}
