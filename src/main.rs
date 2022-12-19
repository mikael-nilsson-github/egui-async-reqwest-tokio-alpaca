//#![allow(dead_code, unused)]
use eframe::egui;
use std::sync::mpsc::{Sender, Receiver};
use reqwest;
use serde_json;
use rand::Rng;

use tokio::runtime::Runtime;

const _KEY_ID: &str = "{APCA-API-KEY-ID}";
const _SECRET_KEY: &str = "{APCA-API-SECRET-KEY}";
const _ACCOUNT_URL: &str = "https://paper-api.alpaca.markets/v2/account";

struct App {
    tx: Sender<u128>,
    rx: Receiver<u128>,
    time_elapsed: u128,
    value: u32,
}

impl Default for App {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        Self {
            tx,
            rx,
            time_elapsed: 0,
            value: 0,
        }
    } 
}

async fn get_account() -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();

    let response: serde_json::Value = client
        .get(_ACCOUNT_URL)
        .header("APCA-API-KEY-ID", _KEY_ID)
        .header("APCA-API-SECRET-KEY", _SECRET_KEY)
        .send()
        .await?
        .json()
        .await?;

    println!("response = {:#?}", response);
    
    Ok(())
}

fn send_request(tx: Sender<u128>, ctx: egui::Context) {
    tokio::spawn(async move {
        let start = std::time::Instant::now(); 

        let duration = rand::thread_rng().gen_range(2000..5000);
        let _ = get_account().await;
        println!("going to sleep for {}ms", duration);
        std::thread::sleep(std::time::Duration::from_millis(duration));
        println!("woke up after {}ms", duration);

        let time_elapsed = start.elapsed();
        let _ = tx.send(time_elapsed.as_millis());
        ctx.request_repaint();
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(msg) = self.rx.try_recv() {
            self.time_elapsed = msg;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("this is a no-freeze gui");
            ui.add(egui::Slider::new(&mut self.value, 1..= 100));

            if ui.button(format!("total time for reqwest + sleep -> {}", self.time_elapsed)).clicked() {
                send_request(self.tx.clone(), ctx.clone());
            };
        });
        println!("{}", self.value);
    }
}

fn main() {
    let run_time = Runtime::new().expect("unable to create tokio runtime");
    let _enter = run_time.enter();
    
    std::thread::spawn(move || {
        run_time.block_on(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
            }
        })
    });

    eframe::run_native(
        "egui + tokio",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(App::default())),
    );
}
