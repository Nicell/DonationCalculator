use std::error::Error;
use chrono::TimeZone;
use chrono_tz::US::Central;
use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Month
    month: u32,

    /// Year
    year: i32,

    /// Pass in token directly
    #[arg(short, long, env = "SHOPIFY_TOKEN", value_name = "TOKEN")]
    token: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Orders {
    orders: Vec<Order>,
}

#[derive(Serialize, Deserialize)]
struct Order {
    line_items: Vec<LineItem>,
    financial_status: String,
    current_subtotal_price: String,
}

#[derive(Serialize, Deserialize)]
struct LineItem {
    name: String,
    price: String,
}

fn fetch_orders(token: &str, start: &str, end: &str) -> Result<Orders, Box<dyn Error>> {
    let url_base = "https://typeractive.myshopify.com";
    let url_endpoint = "/admin/api/2021-07/orders.json";
    let url_params = format!("?status=any&fields=line_items,financial_status,current_subtotal_price&created_at_min={}&created_at_max={}", start, end);
    let url = format!("{}{}{}", url_base, url_endpoint, url_params);

    let result = ureq::get(&url)
        .set("X-Shopify-Access-Token", token)
        .call()?;

    let mut link = parse_link_header::parse_with_rel(result.header("link").unwrap())?;
    let mut next = link.get("next");

    let mut orders: Orders = result.into_json()?;

    while let Some(n) = next {
        let result = ureq::get(&n.raw_uri)
            .set("X-Shopify-Access-Token", token)
            .call()?;
        link = parse_link_header::parse_with_rel(result.header("link").unwrap())?;
        next = link.get("next");
        let mut new_orders: Orders = result.into_json()?;
        orders.orders.append(&mut new_orders.orders);
    }

    Ok(orders)
}

fn main() {
    let cli = Cli::parse();

    let month_end = if cli.month == 12 { 1 } else { cli.month + 1 };
    let year_end = if cli.month == 12 { cli.year + 1 } else { cli.year };

    let token = cli.token.expect("No token provided");

    let start = Central
        .with_ymd_and_hms(cli.year, cli.month, 1, 0, 0, 0)
        .unwrap()
        .to_rfc3339();
    let end = Central
        .with_ymd_and_hms(year_end, month_end, 1, 0, 0, 0)
        .unwrap()
        .to_rfc3339();

    let orders = fetch_orders(&token, &start, &end).unwrap();

    let mut typeractive_donations = 0.0;
    let mut orders_flat = 0;
    let mut orders_percent = 0;
    let mut replacements_refunds = 0;
    let mut customer_donations = 0.0;

    for order in orders.orders {
        if order.financial_status != "refunded" && order.current_subtotal_price != "0.00" {
            let subtotal = order.current_subtotal_price.parse::<f64>().unwrap();
            if subtotal > 100.0 {
                typeractive_donations += subtotal * 0.01;
                orders_percent += 1;
            } else {
                typeractive_donations += 1.0;
                orders_flat += 1;
            }

            for line_item in order.line_items {
                if line_item.name == "Tip" {
                    customer_donations += line_item.price.parse::<f64>().unwrap();
                }
            }
        } else {
            replacements_refunds += 1;
        }
    }

    println!(
        "Total Orders: {}",
        orders_flat + orders_percent + replacements_refunds
    );
    println!("Replacements and Refunds: {}", replacements_refunds);
    println!(
        "Flat donations: {}, Percent Donations: {}",
        orders_flat, orders_percent
    );
    println!("Customer Donations: ${:.2}", customer_donations);
    println!("Typeractive Donation Match: ${:.2}", customer_donations);
    println!("Typeractive Order Donations: ${:.2}", typeractive_donations);
    println!(
        "Typeractive Total Donations: ${:.2}",
        typeractive_donations + customer_donations
    );
    println!(
        "Total Donations: ${:.2}",
        typeractive_donations + customer_donations * 2.0
    );
}
