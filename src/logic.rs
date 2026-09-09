// Bem-vindo ao
// __________         __    __  .__                               __
// \______   \_____ _/  |__/  |_|  |   ____   ______ ____ _____  |  | __ ____
//  |    |  _/\__  \   __\   __\  | _/ __ \ /  ___//    \__  \ |  |/ // __ \
//  |    |   \ / __ \|  |  |  | |  |_\  ___/ \___ \|   |  \/ __ \|    <\  ___/
//  |________/(______/__|  |__| |____/\_____>______>___|__(______/__|__\_____>
//
// ESTE É O ARQUIVO QUE VOCÊ VAI EDITAR. Todo o resto do projeto existe
// só para levar o estado do jogo até as quatro funções abaixo.
//
// Para começar, já deixamos pronta a lógica que impede a sua cobra de andar
// para trás (ela morreria na hora). Os TODOs marcam os próximos passos.
// Documentação: https://docs.battlesnake.com

use crate::models::GameState;
use rand::seq::IndexedRandom;
use serde_json::{json, Value};
use std::collections::HashMap;
use tracing::info;

/// GET / — chamado quando você cadastra a cobra no site e a cada partida.
/// Controla a aparência dela. Opções de cabeça, cauda e cor:
/// https://docs.battlesnake.com/guides/customizations
pub fn info() -> Value {
    info!("INFO");

    json!({
        "apiversion": "1",
        "author": "",          // TODO: coloque aqui o SEU usuário do Battlesnake
        "color": "#8B0000",    // TODO: escolha a cor da sua cobra
        "head": "tiger-king",  // TODO: escolha a cabeça
        "tail": "hook",        // TODO: escolha a cauda
        "version": "1.0.0"
    })
}

/// POST /start — chamado uma vez, quando a partida começa.
/// Bom lugar para preparar qualquer estado inicial.
pub fn start(state: &GameState) {
    info!("JOGO COMEÇOU (partida {})", state.game.id);
}

/// POST /end — chamado uma vez, quando a partida termina.
pub fn end(state: &GameState) {
    info!("FIM DE JOGO após {} turnos", state.turn);
}

/// POST /move — chamado a cada turno. Aqui mora a inteligência da sua cobra.
/// Precisa devolver "up", "down", "left" ou "right".
/// Exemplo do JSON recebido: https://docs.battlesnake.com/api/example-move
pub fn get_move(state: &GameState) -> Value {
    let mut is_move_safe: HashMap<&str, bool> = HashMap::from([
        ("up", true),
        ("down", true),
        ("left", true),
        ("right", true),
    ]);

    // --- Impedir que a cobra ande para trás (já implementado) ---
    // O pescoço é a parte do corpo logo atrás da cabeça. Voltar por cima dele
    // é morte certa, então marcamos aquela direção como insegura.
    let my_head = &state.you.body[0];
    let my_neck = &state.you.body[1];

    if my_neck.x < my_head.x {
        // pescoço à esquerda da cabeça -> não vá para a esquerda
        is_move_safe.insert("left", false);
    } else if my_neck.x > my_head.x {
        // pescoço à direita da cabeça -> não vá para a direita
        is_move_safe.insert("right", false);
    } else if my_neck.y < my_head.y {
        // pescoço abaixo da cabeça -> não desça
        is_move_safe.insert("down", false);
    } else if my_neck.y > my_head.y {
        // pescoço acima da cabeça -> não suba
        is_move_safe.insert("up", false);
    }

    // TODO: Passo 1 — impedir que a cobra saia do tabuleiro
    // let board_width = state.board.width;
    // let board_height = state.board.height;

    // TODO: Passo 2 — impedir que a cobra bata no próprio corpo
    // let my_body = &state.you.body;

    // TODO: Passo 3 — impedir que a cobra bata nas adversárias
    // let opponents = &state.board.snakes;

    // Sobrou alguma direção segura?
    let safe_moves: Vec<&str> = is_move_safe
        .into_iter()
        .filter(|(_, is_safe)| *is_safe)
        .map(|(direction, _)| direction)
        .collect();

    if safe_moves.is_empty() {
        info!("MOVE {}: sem saída! descendo", state.turn);
        return json!({ "move": "down" });
    }

    // Escolhe uma direção segura ao acaso.
    let chosen = safe_moves
        .choose(&mut rand::rng())
        .expect("safe_moves não está vazio");

    // TODO: Passo 4 — ir atrás da comida em vez de sortear, para não morrer de fome
    // let food = &state.board.food;

    info!("MOVE {}: {}", state.turn, chosen);
    json!({ "move": chosen })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Battlesnake, Board, Coord, Game};

    /// Monta um estado de jogo mínimo para os testes, com a cobra deitada
    /// na horizontal: cabeça em `head` e pescoço em `neck`.
    fn game_state(head: Coord, neck: Coord) -> GameState {
        let you = Battlesnake {
            id: "minha-cobra".to_string(),
            name: "MinhaCobra".to_string(),
            health: 100,
            body: vec![head, neck, Coord { x: neck.x, y: neck.y - 1 }],
            head,
            length: 3,
            latency: Some("50".to_string()),
            shout: None,
        };

        GameState {
            game: Game {
                id: "partida-de-teste".to_string(),
                ruleset: HashMap::new(),
                map: Some("standard".to_string()),
                timeout: 500,
            },
            turn: 4,
            board: Board {
                height: 11,
                width: 11,
                food: vec![Coord { x: 5, y: 5 }],
                hazards: vec![],
                snakes: vec![you.clone()],
            },
            you,
        }
    }

    fn chosen_move(state: &GameState) -> String {
        get_move(state)["move"].as_str().unwrap().to_string()
    }

    #[test]
    fn info_devolve_os_campos_obrigatorios() {
        let response = info();

        assert_eq!(response["apiversion"], "1");
        assert!(response.get("author").is_some());
        assert!(response.get("color").is_some());
        assert!(response.get("head").is_some());
        assert!(response.get("tail").is_some());
    }

    #[test]
    fn move_devolve_sempre_uma_direcao_valida() {
        let state = game_state(Coord { x: 5, y: 4 }, Coord { x: 4, y: 4 });

        for _ in 0..50 {
            let direction = chosen_move(&state);
            assert!(
                ["up", "down", "left", "right"].contains(&direction.as_str()),
                "direção inválida: {direction}"
            );
        }
    }

    #[test]
    fn nunca_volta_por_cima_do_pescoco() {
        // pescoço à esquerda da cabeça: "left" seria andar para trás
        let state = game_state(Coord { x: 5, y: 4 }, Coord { x: 4, y: 4 });
        for _ in 0..50 {
            assert_ne!(chosen_move(&state), "left");
        }

        // pescoço à direita da cabeça: "right" seria andar para trás
        let state = game_state(Coord { x: 5, y: 4 }, Coord { x: 6, y: 4 });
        for _ in 0..50 {
            assert_ne!(chosen_move(&state), "right");
        }

        // pescoço abaixo da cabeça: "down" seria andar para trás
        let state = game_state(Coord { x: 5, y: 4 }, Coord { x: 5, y: 3 });
        for _ in 0..50 {
            assert_ne!(chosen_move(&state), "down");
        }

        // pescoço acima da cabeça: "up" seria andar para trás
        let state = game_state(Coord { x: 5, y: 4 }, Coord { x: 5, y: 5 });
        for _ in 0..50 {
            assert_ne!(chosen_move(&state), "up");
        }
    }
}
