# Auto Clicker

Auto clicker próprio, em Rust, com GUI. Funcionalidades:

- CPS configurável (mín/máx) com **randomização (jitter)** para humanizar
- Botão **esquerdo / direito / meio**
- Modo **Toggle** (liga/desliga) ou **Hold** (segurar)
- **Hotkey** global configurável (F1–F12 ou botões laterais do mouse)
- **Delay inicial**, **limite de cliques** e **rajada (burst)**
- **Filtro de janela**: só clica quando a janela com o título indicado está em foco
- **Perfis** salvos em disco (`autoclicker_config.json`, ao lado do `.exe`)
- **CPS em tempo real** + contador de cliques
- Tema claro/escuro

## 1. Instalar o Rust (uma vez)

Baixe e rode o instalador oficial: https://rustup.rs
(No Windows, baixe o `rustup-init.exe`, execute e aceite a instalação padrão.)

Feche e reabra o terminal depois, e confirme:

```
cargo --version
```

## 2. Compilar

Dentro da pasta do projeto:

```
cd C:\Users\Luiz\autoclicker
cargo build --release
```

O executável final fica em:

```
C:\Users\Luiz\autoclicker\target\release\autoclicker.exe
```

Para rodar durante o desenvolvimento (com janela de console para ver erros):

```
cargo run
```

## 3. Usar

1. Abra o `autoclicker.exe`.
2. Ajuste CPS, botão, modo e a hotkey.
3. Aperte a hotkey (padrão **F6**) para ligar/desligar (ou segurar, no modo Hold).

As configurações e perfis são salvos automaticamente ao lado do executável.

## Estrutura do código

| Arquivo          | Responsabilidade                                   |
|------------------|----------------------------------------------------|
| `src/main.rs`    | Inicializa threads e abre a janela                 |
| `src/app.rs`     | Interface gráfica (egui)                            |
| `src/worker.rs`  | Threads de hotkey e de clique + estado compartilhado |
| `src/input.rs`   | WinAPI: enviar clique, ler tecla, janela em foco   |
| `src/config.rs`  | Settings + perfis, salvos em JSON                  |

## Observações

- Alguns jogos/anticheats bloqueiam cliques sintéticos via `SendInput`, ou
  detectam automação. Isto é um projeto pessoal/educacional; use dentro das
  regras do software onde for aplicar.
