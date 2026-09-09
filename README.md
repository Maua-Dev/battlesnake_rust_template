# 🦀 Battlesnake Rust Template

Template de [Battlesnake](https://play.battlesnake.com) em **Rust**, rodando em
**AWS Lambda** com **API Gateway**. O deploy é automático: você programa, dá
push, e o GitHub Actions devolve a URL da sua cobra.

---

## 📦 Pré-requisitos

- **Rust** (edição 2021, versão estável)
  ```bash
  # Windows, Mac ou Linux
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  No Windows você também pode usar o instalador de [rustup.rs](https://rustup.rs).
  Confira com `cargo --version`.

- Noções básicas de **Rust**, **API** e **Lambda**
- **Disposição, competitividade e força de vontade!**

Você **não** precisa instalar Terraform nem AWS CLI: quem cuida do deploy é o CD.

---

## 🚀 Como começar

1. Vá até o repositório [**devmaua_setup**](https://github.com/Maua-Dev/devmaua_setup),
   abra uma **issue** e escolha:
   - **project_name**: `battlesnake_rust_{seu nome}`
   - **project template**: `battlesnake_rust_template`
   - marque o repositório como **público**

2. Aguarde cerca de **1 minuto** e confira em
   [Repositórios da organização](https://github.com/orgs/Maua-Dev/repositories).

3. Clone, compile e rode os testes:
   ```bash
   git clone https://github.com/Maua-Dev/Nome_Do_Seu_Repositorio
   cd Nome_Do_Seu_Repositorio
   cargo build
   cargo test
   ```

   O `cargo test` tem que passar antes de você mexer em qualquer coisa — é o
   mesmo comando que o GitHub Actions roda, e é ele que libera o deploy.

4. Abra [`src/logic.rs`](src/logic.rs) e comece a programar sua cobra 🐍

> 🚀 **O deploy acontece por push na branch `dev`.** Push em qualquer outra
> branch roda só os testes e a compilação, sem tocar na AWS.

---

## 📂 Estrutura do projeto

```
.
├── Cargo.toml                  # dependências e configuração do build
├── src
│   ├── logic.rs                # 👈 É AQUI QUE VOCÊ PROGRAMA
│   ├── models.rs               # structs do estado do jogo (leitura recomendada)
│   └── main.rs                 # ponte com a Lambda — não precisa mexer
├── terraform
│   ├── bootstrap/              # bucket de estado do Terraform
│   └── app/                    # Lambda + API Gateway
└── .github/workflows/CD.yaml   # testes + deploy automático
```

**Você só precisa de `src/logic.rs`.** Os outros arquivos existem para levar o
estado do jogo até as suas quatro funções.

---

## 🧠 As quatro funções

Todas ficam em `src/logic.rs` e recebem um `&GameState` (definido em `models.rs`):

| Função | Rota | Quando é chamada | O que devolve |
|---|---|---|---|
| `info()` | `GET /` | ao cadastrar a cobra e no início de cada partida | aparência (cor, cabeça, cauda) |
| `start(state)` | `POST /start` | uma vez, no começo da partida | nada |
| `get_move(state)` | `POST /move` | **a cada turno** | `{"move": "up" \| "down" \| "left" \| "right"}` |
| `end(state)` | `POST /end` | uma vez, no fim da partida | nada |

A cobra já vem com a lógica que **impede ela de andar para trás**. A partir daí,
os `TODO` em `get_move` marcam os próximos passos:

1. não sair do tabuleiro
2. não bater no próprio corpo
3. não bater nas cobras adversárias
4. ir atrás da comida em vez de sortear a direção

Documentação oficial da API: <https://docs.battlesnake.com/api>

> ⏱️ Você tem cerca de **500 ms** por jogada. Se estourar, o servidor escolhe
> uma direção qualquer por você — normalmente para a morte.

---

## 🧪 Testando

```bash
cargo test
```

O template já vem com testes que garantem que a sua cobra **sempre devolve uma
direção válida** e **nunca volta por cima do próprio pescoço**. Escreva mais
testes conforme for implementando os passos acima.

> 🚨 Os testes rodam no GitHub Actions **antes** do deploy. Se algum falhar, o
> deploy não acontece e a URL da sua cobra não é atualizada.

Vale rodar também:

```bash
cargo clippy   # aponta problemas comuns de Rust
cargo fmt      # formata o código
```

### Rodando um servidor local (opcional)

Se quiser bater na sua cobra com `curl` antes de subir, instale o
[cargo-lambda](https://www.cargo-lambda.info/):

```bash
pip install cargo-lambda
cargo lambda watch
```

Ele sobe um emulador da Lambda na porta 9000. A sua função fica em
`http://localhost:9000/lambda-url/battlesnake` (o `battlesnake` do final é o
nome do pacote no `Cargo.toml`). Em outro terminal:

```bash
curl http://localhost:9000/lambda-url/battlesnake/
```

```bash
curl -X POST http://localhost:9000/lambda-url/battlesnake/move -H 'Content-Type: application/json' -d '{"turn":1,"game":{"id":"1","ruleset":{},"timeout":500},"board":{"width":11,"height":11,"food":[],"snakes":[]},"you":{"id":"s1","name":"eu","health":100,"body":[{"x":5,"y":4},{"x":4,"y":4}],"head":{"x":5,"y":4},"length":2}}'
```

Um JSON completo de exemplo está em <https://docs.battlesnake.com/api/example-move>.

---

## ☁️ Deploy

O deploy é disparado por push na branch **`dev`**:

```bash
git add .
git commit -m "minha cobra agora desvia das paredes"
git push origin dev
```

O que o CD faz, nessa ordem:

1. **ExecuteTests** — roda `cargo test`
2. **Bootstrap** — garante o bucket S3 que guarda o estado do Terraform
3. **build_rust** — compila para `x86_64-unknown-linux-musl` e empacota o
   binário `bootstrap` num zip
4. **deploy_app** — `terraform apply`, criando a Lambda e o API Gateway

No fim, o resumo da execução mostra a **URL da sua cobra** e um link para os
logs no CloudWatch. Você também encontra a URL no output `api_url_base` do
passo *Terraform Apply*.

> Rust não tem runtime gerenciado na Lambda. Por isso compilamos um binário
> estático chamado `bootstrap` e subimos no runtime `provided.al2023`. O
> resultado é um cold start bem mais rápido que Java ou Python.

---

## 🎯 Cadastrando no Battlesnake

1. Entre em [play.battlesnake.com](https://play.battlesnake.com)
2. **My Battlesnakes** → **Create Battlesnake**
3. No campo **URL**, cole a URL do deploy
   (algo como `https://abc123.execute-api.us-east-1.amazonaws.com/dev`)
4. Salve e mande ver nos jogos e desafios!

Se o site reclamar da URL, teste antes no terminal:

```bash
curl https://SUA_URL_AQUI/
```

Deve responder o JSON do `info()`.

---

## 📌 Observações

- Toda a lógica da partida vive em `get_move()`.
- **Evite adicionar dependências pesadas.** O zip da Lambda tem limite de 50 MB
  e cada dependência aumenta o tempo de compilação no CI.
- Os logs ficam no **CloudWatch**, com retenção de 14 dias. Use `tracing::info!`
  (já importado em `logic.rs`) em vez de `println!` para que eles apareçam
  formatados.
- A branch de deploy é **`dev`**. Push em outras branches roda só os testes.

---

## 🛠 Ferramentas úteis

- [Battlesnake Docs](https://docs.battlesnake.com/) — documentação da API
- [The Rust Book](https://doc.rust-lang.org/book/) — o livro oficial de Rust
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — Rust na prática
- [serde_json](https://docs.rs/serde_json/) — manipulação de JSON
- [cargo-lambda](https://www.cargo-lambda.info/) — rodar a Lambda localmente

---

## 📞 Fale com a gente

Dúvidas? Chama no [Discord](https://discord.gg/Yr2VPgAmcb) da Dev. Community Mauá.
