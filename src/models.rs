//! Estruturas que representam o estado do jogo enviado pelo Battlesnake.
//!
//! Elas espelham exatamente o corpo JSON das requisições descritas em
//! https://docs.battlesnake.com/api
//!
//! Você não precisa mexer neste arquivo, mas vale a pena ler: é o mapa
//! completo de tudo que a sua cobra consegue "enxergar" a cada turno.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Uma posição no tabuleiro. A origem (0, 0) fica no canto inferior esquerdo:
/// x cresce para a direita e y cresce para cima.
#[derive(Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

/// Uma cobra em jogo — pode ser a sua (`GameState::you`) ou uma adversária
/// (dentro de `Board::snakes`).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Battlesnake {
    pub id: String,
    pub name: String,
    /// Vai de 0 a 100. Chegou a 0, a cobra morre de fome.
    pub health: i32,
    /// Corpo inteiro, da cabeça (índice 0) até a cauda (último índice).
    pub body: Vec<Coord>,
    pub head: Coord,
    pub length: i32,
    #[serde(default)]
    pub latency: Option<String>,
    #[serde(default)]
    pub shout: Option<String>,
}

/// O tabuleiro no turno atual.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Board {
    pub height: i32,
    pub width: i32,
    /// Comidas disponíveis. Comer devolve a vida para 100 e aumenta o corpo em 1.
    pub food: Vec<Coord>,
    /// Casas perigosas (só aparecem em alguns modos de jogo).
    #[serde(default)]
    pub hazards: Vec<Coord>,
    /// Todas as cobras vivas, incluindo a sua.
    pub snakes: Vec<Battlesnake>,
}

/// Metadados da partida (regras, tempo limite, etc.).
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Game {
    pub id: String,
    pub ruleset: HashMap<String, Value>,
    #[serde(default)]
    pub map: Option<String>,
    /// Tempo máximo, em milissegundos, para responder o /move.
    pub timeout: u32,
}

/// O pacote completo que chega em /start, /move e /end.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GameState {
    pub game: Game,
    pub turn: i32,
    pub board: Board,
    pub you: Battlesnake,
}
