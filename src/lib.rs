#![allow(clippy::wildcard_imports, dead_code)]

use seed::{prelude::*, *};

const ENTER_KEY: u32 = 13;
const BUTTON_DEFAULT: &str = "CONNECT WALLET!";
const BUTTON_CONNECTED: &str = "CONNECTED!";
const BUTTON_ERROR: &str = "COULD NOT CONNECT!";
const STATUS_DEFAULT: &str = " is-info";
const STATUS_SUCCESS: &str = " is-success";
const STATUS_ERROR: &str = " is-danger";

enum Msg {
    Connect,
    WalletConnection(Result<JsValue, JsValue>),
    FetchError(FetchError),
    ChangePubkey(String),
    SearchPubkey,
    ProcessTokenList(Token, Vec<Token>),
}

struct Model {
    button_text: String,
    notification_text: String,
    wallet_status: String,
    pubkey: String,
    token_list: Vec<Token>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Token {
    token_name: String,
    token_icon: String,
    token_amount: TokenAmount,
    #[serde(default)]
    price_usdt: f64,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TokenAmount {
    ui_amount: f64,
}

fn init(_: Url, _: &mut impl Orders<Msg>) -> Model {
    Model {
        button_text: BUTTON_DEFAULT.to_string(),
        notification_text: String::new(),
        wallet_status: STATUS_DEFAULT.to_string(),
        pubkey: String::new(),
        token_list: Vec::new(),
    }
}

fn update(msg: Msg, model: &mut Model, orders: &mut impl Orders<Msg>) {
    match msg {
        Msg::Connect => {
            orders.perform_cmd(async { Msg::WalletConnection(connect_to_wallet().await) });
        }
        Msg::WalletConnection(Ok(pubkey)) => match pubkey.as_string() {
            Some(p) => {
                model.button_text = BUTTON_CONNECTED.to_string();
                model.wallet_status = STATUS_SUCCESS.to_string();
                model.notification_text = "".to_string();
                model.pubkey = p;
                orders.send_msg(Msg::SearchPubkey);
            }
            None => {
                model.wallet_status = STATUS_ERROR.to_string();
                model.button_text = BUTTON_ERROR.to_string();
                model.notification_text =
                    "Are you sure you have installed the phantom wallet?".to_string();
            }
        },
        Msg::WalletConnection(Err(error)) => {
            model.notification_text = format!("{:?}", error);
            model.wallet_status = STATUS_ERROR.to_string();
            model.button_text = BUTTON_ERROR.to_string();
        }
        Msg::FetchError(e) => {
            model.notification_text = format!("{:?}", e);
            model.wallet_status = STATUS_ERROR.to_string();
        }
        Msg::ChangePubkey(s) => {
            model.pubkey = s;
        }
        Msg::SearchPubkey => {
            let pubkey = model.pubkey.clone();
            let pubkey = pubkey;
            orders.perform_cmd(async move { get_wallet_info(pubkey).await });
        }
        Msg::ProcessTokenList(sol, tokens) => {
            let sol = Token {
                token_name: sol.token_name,
                price_usdt: sol.price_usdt * sol.token_amount.ui_amount,
                token_amount: TokenAmount {
                    ui_amount: (sol.token_amount.ui_amount * 100.0).round() / 100.0,
                },
                token_icon: sol.token_icon,
            };
            let mut tokens = tokens
                .into_iter()
                .filter_map(|mut x| {
                    if x.token_name == "".to_string() {
                        return None;
                    }
                    x.price_usdt = x.price_usdt * x.token_amount.ui_amount;
                    x.price_usdt = (x.price_usdt * 100.0).round() / 100.0;
                    x.token_amount.ui_amount = (x.token_amount.ui_amount * 100.0).round() / 100.0;
                    return Some(x);
                })
                .collect::<Vec<Token>>();
            tokens.sort_by(|a, b| (b.price_usdt).partial_cmp(&(a.price_usdt)).unwrap());
            model.token_list = vec![sol];
            model.token_list.append(&mut tokens);
        }
    }
}

async fn get_wallet_info(pubkey: String) -> Msg {
    async fn fetch_wrapper(pubkey: String) -> Result<(Token, Vec<Token>), FetchError> {
        #[derive(serde::Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct SolResponse {
            #[serde(default)]
            lamports: u64,
        }

        let sol_url = "https://public-api.solscan.io/account/".to_owned() + pubkey.as_str();
        let sol_response = fetch(Request::new(sol_url));
        let token_url =
            "https://public-api.solscan.io/account/tokens?account=".to_owned() + pubkey.as_str();
        let token_response = fetch(Request::new(token_url));

        let (sol_response, token_response) = futures::join!(sol_response, token_response);
        let sol_response = sol_response?;
        let token_response = token_response?;

        let sol = sol_response.check_status()?.json::<SolResponse>().await?;
        let sol = Token {
            token_name: "SOL".to_string(),
            token_amount: TokenAmount {
                ui_amount: sol.lamports as f64 * 0.000000001,
            },
            price_usdt: 0.0,
            token_icon: "https://cryptorank-images.s3.eu-central-1.amazonaws.com/coins/solana1606979093056.png".to_string(),
        };
        return Ok((
            sol,
            token_response.check_status()?.json::<Vec<Token>>().await?,
        ));
    }

    match fetch_wrapper(pubkey).await {
        Ok((s, v)) => return Msg::ProcessTokenList(s, v),
        Err(e) => return Msg::FetchError(e),
    }
}

fn view(model: &Model) -> Node<Msg> {
    div![section![
        C!["is-medium", "ml-6"],
        div![
            C!["hero-body"],
            div![
                C!["columns is-mobile is-centered"],
                div![
                    C!["column is-5"],
                    view_intro(),
                    view_wallet(
                        model.button_text.clone(),
                        model.notification_text.clone(),
                        model.wallet_status.clone()
                    ),
                ]
            ],
            div![
                C!["columns is-mobile is-centered"],
                hr![],
                div![
                    C!["column is-5"],
                    search_pubkey_input(&model.pubkey),
                    token_table(&model.token_list),
                ]
            ]
        ]
    ],]
}

fn view_intro() -> Node<Msg> {
    div![
        div![
            C!["subtitle has-text-centered is-4"],
            "Seed-Phantom Example",
        ],
        div![
            C!["text has-text-centered"],
            raw![
                "This page is an example-implementation of
                <a href='https://seed-rs.org/' target='_blank'>seed-rs</a>
                and the <a href='https://docs.phantom.app/' target='_blank'>phantom.app</a> wallet,
            showcasing how to connect ot the wallet and also how to query blockchain information.
            Seed-rs is a <a href='https://www.rust-lang.org/' target='_blank'>rust</a> framework that 
            runs on <a href='https://rustwasm.github.io/docs/book/' target='_blank'>WASM</a> (WebAssembly).
            The Phantom App is the most popular wallet for 
            <a href='https://docs.solana.com/' target='_blank'>Solana</a>, which is a modern
            and very fast blockchain."
            ],
            div![]
        ],
        br![],
        hr![],
    ]
}

fn view_wallet(button_text: String, notification_text: String, status: String) -> Node<Msg> {
    div![
        div![
            C!["columns"],
            div![C!["column"],],
            a![
                C!["column"],
                attrs! {At::Href => "https://github.com/Gheo-Tech/rust-phantom-poc",
                At::Target => "_blank"},
                button![C!["button fa-align-center is-link"], "Source Code",],
            ],
            div![
                C!["column"],
                button![
                    C!["button fa-align-center".to_string() + &status],
                    button_text,
                    ev(Ev::Click, |_| Msg::Connect),
                ],
            ],
            div![C!["column"],],
        ],
        div![
            C!["columns"],
            div![
                C!["column is-full"],
                wallet_notification(notification_text.clone(), status.clone()),
            ],
        ]
    ]
}

fn wallet_notification(text: String, status: String) -> Node<Msg> {
    if text != "".to_string() {
        return div![
            C!["notification has-text-centered".to_string() + &status],
            text,
        ];
    }
    div![]
}

fn search_pubkey_input(m: &String) -> Node<Msg> {
    div![
        C!["control"],
        input![
            C!["input",],
            attrs! {
                At::Placeholder => "Search for any solana key...";
                At::Value => m.clone(),
            },
            keyboard_ev(Ev::KeyDown, |keyboard_event| {
                IF!(keyboard_event.key_code() == ENTER_KEY => Msg::SearchPubkey)
            }),
            input_ev(Ev::Input, Msg::ChangePubkey),
        ]
    ]
}

fn token_table(tokens: &Vec<Token>) -> Node<Msg> {
    if tokens.len() == 0 {
        return div![];
    }
    div![
        C!["container"],
        hr![],
        div![tokens.iter().map(|t| view_token(t)),],
    ]
}

fn view_token(t: &Token) -> Node<Msg> {
    let mut amount_and_price = t.token_amount.ui_amount.to_string().clone();
    if t.price_usdt > 0.0 {
        amount_and_price += &" ($".to_string();
        amount_and_price += &t.price_usdt.to_string();
        amount_and_price += &")".to_string();
    }
    div![
        C!["columns"],
        div![
            C!["column is-one-quarter"],
            img![attrs! {
                At::Src => t.token_icon.clone(),
                At::Width => 50,
                At::Height => 50,
            },],
        ],
        div![
            C!["column"],
            div![C!["has-text-weight-bold"], t.token_name.clone()],
            div![amount_and_price],
        ],
    ]
}

#[wasm_bindgen(start)]
pub fn start() {
    App::start("app", init, update, view);
}

#[wasm_bindgen(module = "/js/solana.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn connect_to_wallet() -> Result<JsValue, JsValue>;
}
