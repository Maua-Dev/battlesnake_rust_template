//! Ponte entre o AWS Lambda e a lógica da sua cobra.
//!
//! Você NÃO precisa mexer aqui. Este arquivo recebe o evento do API Gateway,
//! descobre qual rota do Battlesnake foi chamada e repassa para `logic.rs`.
//!
//! Rotas da API (https://docs.battlesnake.com/api):
//!   GET  /        -> aparência da cobra
//!   POST /start   -> a partida começou
//!   POST /move    -> escolha a jogada deste turno
//!   POST /end     -> a partida acabou

mod logic;
mod models;

use lambda_http::{run, service_fn, Body, Error, Request, RequestPayloadExt, Response};
use models::GameState;
use serde_json::{json, Value};
use tracing::error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Configura os logs no formato que o CloudWatch entende. O nível vem da
    // variável RUST_LOG (definida no Terraform); o padrão é "info".
    lambda_http::tracing::init_default_subscriber();

    run(service_fn(handler)).await
}

async fn handler(event: Request) -> Result<Response<Body>, Error> {
    match extract_route(event.uri().path()) {
        "/start" => match parse_state(&event) {
            Ok(state) => {
                logic::start(&state);
                text_response(200, "ok")
            }
            Err(reason) => bad_request(reason),
        },

        "/move" => match parse_state(&event) {
            Ok(state) => json_response(200, logic::get_move(&state)),
            Err(reason) => bad_request(reason),
        },

        "/end" => match parse_state(&event) {
            Ok(state) => {
                logic::end(&state);
                text_response(200, "ok")
            }
            Err(reason) => bad_request(reason),
        },

        // A raiz — e qualquer caminho desconhecido — devolve as informações
        // da cobra, que é o que o site do Battlesnake espera de um GET /.
        _ => json_response(200, logic::info()),
    }
}

/// Descobre qual rota do Battlesnake foi chamada.
///
/// O API Gateway entrega o caminho com o nome do stage na frente
/// (ex.: "/dev/move" em vez de "/move"), por isso comparamos pelo final do
/// caminho em vez de exigir igualdade exata.
fn extract_route(path: &str) -> &'static str {
    let path = path.trim_end_matches('/');

    for route in ["/start", "/move", "/end"] {
        if path.ends_with(route) {
            return route;
        }
    }

    "/"
}

/// Lê o corpo da requisição como um `GameState`. Devolve o motivo da falha em
/// texto, para irmos direto ao ponto quando algo vier fora do esperado.
fn parse_state(event: &Request) -> Result<GameState, String> {
    match event.json::<GameState>() {
        Ok(Some(state)) => Ok(state),
        Ok(None) => Err("o corpo da requisição veio vazio".to_string()),
        Err(e) => Err(format!("JSON inválido: {e:?}")),
    }
}

fn json_response(status: u16, body: Value) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(body.to_string()))?)
}

fn text_response(status: u16, body: &str) -> Result<Response<Body>, Error> {
    Ok(Response::builder()
        .status(status)
        .header("Content-Type", "text/plain")
        .body(Body::from(body.to_owned()))?)
}

/// O corpo da requisição não era um estado de jogo válido. Devolvemos 400 com
/// o motivo, que aparece no CloudWatch para você depurar.
fn bad_request(reason: String) -> Result<Response<Body>, Error> {
    error!("não foi possível ler o estado do jogo: {reason}");
    json_response(400, json!({ "error": reason }))
}

#[cfg(test)]
mod tests {
    use super::{extract_route, handler};
    use lambda_http::{http, Body, Request};

    /// Um estado de jogo válido, no mesmo formato que o servidor do
    /// Battlesnake envia. Veja https://docs.battlesnake.com/api/example-move
    const GAME_STATE: &str = r#"{
        "game": {"id": "g1", "ruleset": {"name": "standard"}, "map": "standard", "timeout": 500},
        "turn": 4,
        "board": {
            "height": 11, "width": 11,
            "food": [{"x": 5, "y": 5}], "hazards": [],
            "snakes": []
        },
        "you": {
            "id": "s1", "name": "MinhaCobra", "health": 100,
            "body": [{"x": 5, "y": 4}, {"x": 4, "y": 4}, {"x": 3, "y": 4}],
            "head": {"x": 5, "y": 4}, "length": 3, "latency": "50", "shout": ""
        }
    }"#;

    fn request(method: &str, path: &str, body: &str) -> Request {
        http::Request::builder()
            .method(method)
            .uri(format!("https://abc123.execute-api.us-east-1.amazonaws.com{path}"))
            .header("Content-Type", "application/json")
            .body(Body::from(body.to_owned()))
            .unwrap()
    }

    #[test]
    fn reconhece_as_rotas_sem_o_stage() {
        assert_eq!(extract_route("/"), "/");
        assert_eq!(extract_route("/start"), "/start");
        assert_eq!(extract_route("/move"), "/move");
        assert_eq!(extract_route("/end"), "/end");
    }

    #[test]
    fn reconhece_as_rotas_com_o_stage_do_api_gateway() {
        assert_eq!(extract_route("/dev"), "/");
        assert_eq!(extract_route("/dev/"), "/");
        assert_eq!(extract_route("/dev/start"), "/start");
        assert_eq!(extract_route("/dev/move"), "/move");
        assert_eq!(extract_route("/dev/end"), "/end");
    }

    #[tokio::test]
    async fn responde_as_quatro_rotas_do_battlesnake() {
        let casos = [
            ("GET", "/dev", ""),
            ("POST", "/dev/start", GAME_STATE),
            ("POST", "/dev/move", GAME_STATE),
            ("POST", "/dev/end", GAME_STATE),
        ];

        for (method, path, body) in casos {
            let response = handler(request(method, path, body)).await.unwrap();
            assert_eq!(response.status(), 200, "{method} {path} não devolveu 200");
        }
    }

    #[tokio::test]
    async fn move_devolve_uma_direcao_valida() {
        let response = handler(request("POST", "/dev/move", GAME_STATE)).await.unwrap();
        let body = std::str::from_utf8(response.body().as_ref()).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(body).unwrap();

        let direction = parsed["move"].as_str().expect("faltou o campo move");
        assert!(["up", "down", "left", "right"].contains(&direction));
    }

    #[tokio::test]
    async fn corpo_invalido_devolve_400() {
        let response = handler(request("POST", "/dev/move", "{isso não é json}"))
            .await
            .unwrap();

        assert_eq!(response.status(), 400);
    }
}
