# DzkLP - Clicker — Resumo do projeto

Documento de retomada. Atualizado em 11/11/2026.

## O que é

Autoclicker próprio, em **Rust**, com interface gráfica (egui/eframe). Clica no
**nível do sistema operacional** (Windows `SendInput`). É uma ferramenta de SO,
não injeta em jogo nenhum.

- **Projeto:** `C:\Users\Luiz\autoclicker\`
- **Executável (dev):** `C:\Users\Luiz\autoclicker\target\release\autoclicker.exe`
- **Executável (pronto p/ usar, com ícone):** `C:\Users\Luiz\Desktop\DzkLP-Clicker.exe`
- O `.exe` é autossuficiente (CRT estático) — roda em qualquer Windows sem instalar nada.

## Funcionalidades já prontas

- CPS configurável de 1 a 100, com **jitter** (variação entre mín e máx)
- Botão **esquerdo / direito / meio**
- Modos: **Toggle** (liga/desliga), **Segurar tecla**, **Segurar mouse esquerdo**
- **Captura de hotkey**: clica no campo "Tecla", aperta qualquer tecla, ele reconhece
  (F1–F24, letras, números, setas, numpad, Mouse 4/5). Esc cancela.
- **Atraso ao iniciar**, **limite de cliques**, **rajada (burst)**
- **Seletor de janela aberta**: só clica quando a janela escolhida está em foco
- **Perfis** salvos em disco (`autoclicker_config.json`, ao lado do .exe)
- **Timing de alta precisão** (spin nos últimos ms → o CPS real bate com o configurado)
- **Interface**: janela grande (960x600) em 2 colunas, tema claro/escuro
- **Nome** da janela: "DzkLP - Clicker"
- **Logo "D"** (quadrado âmbar) no cabeçalho, no ícone da janela, na barra de tarefas
  e **embutida no próprio .exe** (aparece no Explorer)

## Estrutura do código

| Arquivo          | Responsabilidade                                        |
|------------------|---------------------------------------------------------|
| `src/main.rs`    | Sobe as threads, define janela/ícone, abre a GUI        |
| `src/app.rs`     | Interface gráfica (egui), cards, captura de hotkey      |
| `src/theme.rs`   | Paleta (ardósia + âmbar), estilos, cartões              |
| `src/icon.rs`    | Gera o ícone "D" em runtime (janela/taskbar)            |
| `src/worker.rs`  | Threads de hotkey e de clique + estado compartilhado    |
| `src/input.rs`   | WinAPI: enviar clique, ler tecla, listar/foco de janela |
| `src/config.rs`  | Settings + perfis, salvos em JSON                       |
| `build.rs`       | Embute `assets/icon.ico` no .exe                        |
| `assets/icon.ico`| Ícone "D" multi-tamanho                                 |

## Como compilar

Rust já está instalado em `C:\Users\Luiz\.cargo\bin\`.

```
cd C:\Users\Luiz\autoclicker
cargo build --release
```

Para gerar o .exe portátil (runtime embutido), compilar com:

```
$env:RUSTFLAGS="-C target-feature=+crt-static"
cargo build --release
Remove-Item Env:\RUSTFLAGS
```

Dica: se `cargo` não for reconhecido no terminal, use o caminho completo
`& "$env:USERPROFILE\.cargo\bin\cargo.exe"` ou reabra o PowerShell.

## Pendentes (melhorias do autoclicker)

Ideias levantadas, ainda não feitas:

- [ ] Renomear o binário de build para `DzkLP-Clicker.exe` direto (hoje a cópia é manual)
- [ ] Minimizar para a **bandeja** (system tray)
- [ ] Mostrar a **tecla atual** também no card de status
- [ ] Opção de **layout em 3 colunas** / ajustes finos de interface
- [ ] Janela **lembrar tamanho/posição**
- [ ] Jitter de **posição** do cursor (mover alguns pixels por clique)

## Fora de escopo (combinado)

O changelog do Evade/painel descreve um **cliente de cheat de Minecraft** que
**injeta no cliente do jogo** (Lunar/Forge/CheatBreaker) com módulos de combate
(AutoArmor, AutoRefill, AutoDefend, hitbox/reach), movimento (NoSlow, SnapTap) e
visuals/ESP (vida do inimigo, ItemInfo). **Isso não será construído aqui** — é
desenvolvimento de hacked client / vantagem em multiplayer, fora do que eu ajudo,
independente do que o servidor tolera no momento. O assunto ficou de "conversar
depois", mas essa é a posição.

O que segue sendo bem-vindo: evoluir o **autoclicker de nível de SO** (funções,
usabilidade, interface).
