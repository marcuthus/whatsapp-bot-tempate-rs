# WhatsApp Bot Template (Rust) 🦀

Um template robusto e moderno para criação de bots de WhatsApp utilizando **Rust**, **whatsapp-rust** e as melhores práticas de desenvolvimento. Inspirado no [whatsapp-bot-template](../whatsapp-bot) em TypeScript.

## ✨ Funcionalidades

- 🦀 **Rust + Tokio**: tipagem forte, segurança de memória e performance.
- 💬 **whatsapp-rust**: cliente WhatsApp Web multi-device em Rust puro, com sessão persistida em SQLite (`auth/whatsapp.db`), login por QR code ou código de pareamento e reconexão automática.
- ⚡ **Comandos tipados**: enum `Command` com parser próprio (prefixo `!`), argumentos tipados e erros de argumento reportados ao usuário.
- 📂 **Organização modular**: cada categoria de comando e cada evento vive no seu próprio módulo.
- 📝 **Logs com `tracing`**: console colorido e arquivos em `logs/` (`combined.log` e `error.log`).
- 🎨 **Qualidade de código**: `rustfmt` (4 espaços) e `clippy` com `-D warnings`, validados no CI.
- ⚙️ **Gestão de variáveis**: `.env` via `dotenvy`, validado na inicialização com erro claro quando falta alguma variável.
- 🔐 **Checks de permissão**: exemplo de comando restrito ao dono do bot (`!uptime`).

## 🛠️ Pré-requisitos

- [Rust](https://www.rust-lang.org/tools/install) estável (edition 2024, Rust 1.94 ou superior)
- Um número de WhatsApp para vincular ao bot

## 🚀 Instalação e Início

1.  **Clone o repositório:**

    ```bash
    git clone https://github.com/seu-usuario/whatsapp-bot-rs.git
    cd whatsapp-bot-rs
    ```

2.  **Configure o ambiente:**
    Copie o arquivo `.env.example` para `.env` e preencha as variáveis:

    ```env
    # Número do dono, com código do país e sem símbolos
    WHATSAPP_ADMIN_NUMBER=5511999999999
    # Opcional: se definido, o bot gera um código de pareamento além do QR code
    WHATSAPP_PAIRING_NUMBER=
    ```

3.  **Inicie o bot:**

    ```bash
    # Modo desenvolvimento
    cargo run

    # Build para produção
    cargo build --release
    ./target/release/whatsapp-bot-rs
    ```

4.  **Vincule o WhatsApp:** na primeira execução, escaneie o QR code exibido no terminal em _Aparelhos conectados_ (ou digite o código de pareamento). A sessão fica salva em `auth/`; apague a pasta para vincular outro número.

> A primeira compilação é demorada (muitas dependências criptográficas e de protocolo).

## 💡 Comandos de exemplo

O prefixo padrão é `!` (veja `src/helpers/config.rs`).

| Comando         | Descrição                                |
| --------------- | ---------------------------------------- |
| `!ping`         | Mostra o ping do bot.                    |
| `!help`         | Lista os comandos.                       |
| `!menu`         | Menu de comandos por categoria.          |
| `!id [@pessoa] [private-reply]` | Mostra o seu ID ou o da pessoa mencionada. `private-reply` (`true` por padrão) responde no privado. |
| `!members [private-reply]` | Lista os membros do grupo e seus IDs. `private-reply` (`true` por padrão) envia a lista no privado. |
| `!group-id`     | Mostra o ID do grupo.                    |
| `!ban-member @pessoa` | Bane um membro do grupo (apenas admins). |
| `!add-member [número]` | Adiciona um número ao grupo, ou todos os contatos da mensagem respondida; informa no final quais falharam (apenas admins). |
| `!invite [número]` | Link de convite do grupo, ou envia no privado (apenas admins). |
| `!leave`        | O bot sai do grupo (apenas admins).      |
| `!ban-all-members confirm` | Bane todos os membros que não são admins (apenas admins; exige `confirm`). |
| `!everyone [mensagem]` | Menciona todos os membros do grupo (apenas admins). |
| `!create-group <nome>` | Cria um grupo, adiciona você como admin e o trava para admins (apenas dono). |
| `!sticker [crop] [center]` | Cria uma figurinha a partir de uma imagem, GIF ou vídeo (legenda ou resposta); vídeos e GIFs viram sticker animado (até 10 s, requer o `ffmpeg`). `crop`: `true` (padrão, preenche e corta), `false` (inteira) ou `dented` (amassada); `center`: `true` (padrão) ou `false`. |
| `!sticker2media` | Transforma um sticker em imagem ou, se for animado, em vídeo (responda o sticker com o comando). Requer o `ffmpeg` para stickers animados. |
| `!profile-photo [@pessoa]` | Envia a sua foto de perfil ou a da pessoa mencionada. |
| `!sum <a> <b>`  | Soma dois números (exemplo).             |
| `!uptime`       | Tempo online do bot (apenas dono).       |

## 📜 Comandos Úteis

| Comando                                     | Descrição                                 |
| ------------------------------------------- | ----------------------------------------- |
| `cargo run`                                 | Executa o bot em modo de desenvolvimento. |
| `cargo build --release`                     | Compila o bot otimizado para produção.    |
| `cargo fmt`                                 | Formata o código.                         |
| `cargo clippy --all-targets -- -D warnings` | Verifica erros de linting.                |
| `cargo test`                                | Executa os testes.                        |

Defina `RUST_LOG=debug` para ver logs mais detalhados no console.

## 📁 Estrutura de Pastas

```text
src/
├── main.rs          # Ponto de entrada
├── client.rs        # Criação do bot, login e ciclo de vida
├── data.rs          # Data, Context e Error compartilhados
├── error.rs         # Handler global de erros
├── commands/        # Enum Command, parser e implementação por categoria
│   ├── info/
│   ├── example/
│   └── admin/
├── events/          # Handlers de eventos (QR code, pareamento, conexão, logout)
├── helpers/         # Auxiliares (config e logger)
│   ├── config.rs    # Variáveis de ambiente (Config)
│   └── logger.rs    # Configuração do tracing (console + arquivos)
└── utils/           # Utilitários (checks de permissão, etc.)
```

## ➕ Adicionando um comando

1. Adicione uma variante ao enum `Command` em `src/commands/mod.rs`, uma entrada em `DESCRIPTIONS`, o ramo em `Command::from_parts` (os argumentos são lidos em ordem) e o nome em `Command::name`.
2. Crie `src/commands/<categoria>/<comando>.rs` com `pub async fn run(ctx: &Context, ...)` e declare o módulo no `mod.rs` da categoria.
3. Despache a variante no `match` de `commands::run`.

> Diferente do template em TypeScript, Rust não carrega módulos em tempo de execução, e a validação de argumentos é feita pelo parser do enum `Command` em vez do Argus (que é uma biblioteca npm).

## ➕ Tratando um novo evento

Crie `src/events/<evento>.rs` com uma `pub async fn run(...)` e registre-a no `Bot::builder()` em `src/client.rs` (`on_connected`, `on_logged_out`, `on_event`, ...).

## ⚠️ Aviso

O WhatsApp não oferece uma API oficial para este tipo de cliente. O uso para spam ou automação abusiva pode resultar em banimento do número. Use com responsabilidade.

## 🐳 Docker

O projeto inclui um `Dockerfile` (multi-stage, executa como usuário sem privilégios) e um `docker-compose.yml`. As variáveis são lidas do arquivo `.env` (copie o `.env.example`); o `.env` nunca entra na imagem.

Na primeira execução é preciso vincular o WhatsApp, então suba o container **anexado** para ver o QR code (ou o código de pareamento) nos logs. Depois de pareado, a sessão fica no volume `auth` e você pode rodar em segundo plano:

```bash
docker compose up --build
# depois de vincular, pare com Ctrl+C e rode em segundo plano:
docker compose up -d
```

Os volumes `logs` (logs do bot) e `auth` (sessão do WhatsApp) são persistentes. Para vincular outro número: `docker compose down -v`.

## ⚖️ Licença

Este projeto está sob a licença MIT. Veja o arquivo [LICENSE](LICENSE) para mais detalhes.

---

Feito com ❤️ por [Marcuth](https://github.com/marcuth)
