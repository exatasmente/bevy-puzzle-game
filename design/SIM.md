# SIM — desenho

Um jogo curto que termina numa pergunta, e que **outras pessoas** podem montar
para os seus próprios pedidos de namoro ou casamento.

Nome de trabalho: `SIM`. Alternativas consideradas: `PEDIDO`, `DIZ SIM`,
`A CONSTELACAO`.

Status: desenho para aprovação. Nada implementado.

---

## 1. O que é, em uma frase

Quem vai pedir escreve os momentos que quer citar, escolhe uma figura e a
pergunta; o app devolve um link. Quem recebe abre o link e liga estrelas com o
dedo — cada estrela acende um dos momentos — e quando a última liga na primeira
descobre que estava desenhando um coração, um anel, as duas iniciais. Aí vem a
pergunta.

## 2. Os dois usuários

Este é o ponto que separa este projeto de tudo que já existe no repo: o Color
Puzzle tem um usuário, este tem dois, e eles nunca veem a mesma tela.

| | **Autor** (quem pede) | **Convidado** (quem recebe) |
| --- | --- | --- |
| Vê | o editor, o ensaio, o link | só o jogo, uma vez |
| Faz | escreve, ensaia, entrega | arrasta o dedo, responde |
| Pode falhar? | pode errar e corrigir à vontade | **não pode falhar nunca** |

A regra de ouro do produto está na última linha: **o app não tem estado de
falha.** Nada de vidas, nada de cronômetro que encerra, nada de placar, nada de
"Fim de Jogo". Tudo que o Color Puzzle é, este não é. Um pedido interrompido por
uma tela de derrota é um pedido estragado, e não há como pedir de novo.

## 3. Distribuição sem servidor

Tudo que o autor escreveu viaja **no fragmento da URL**:

```
https://<usuario>.github.io/bevy-puzzle-game/#p=<base64url>
```

Fragmento porque o fragmento **nunca é enviado ao servidor**. O conteúdo é
pessoal — nomes, datas, a pergunta — e assim ele não passa pelos logs do GitHub,
não vai pra banco nenhum, não existe em lugar nenhum além do link. Isso é ao
mesmo tempo uma necessidade técnica (o Pages é estático, não há backend) e uma
promessa que vale escrever na tela do editor: *nada disso sai do seu celular;
está todo dentro do link*.

Sem fragmento, o app abre o menu normal do Color Puzzle, com um cartão novo.
Com fragmento, ele **não passa pelo menu** — vai direto para o pedido.

### 3.1 Os dois jeitos de entregar

**Presencial — o principal.** O autor abre no próprio celular, com tudo já
carregado, e entrega na mão. É assim que pedido acontece: vocês estão juntos. O
tamanho do download é irrelevante aqui, e o service worker que já existe no repo
(`docs/sw.js`) garante que funciona **sem sinal nenhum** depois da primeira
visita — restaurante com wi-fi ruim, praia, avião.

**Link — o secundário.** Manda no WhatsApp. Aqui está o maior risco do produto,
e ele é medido: `docs/puzzle_wasm_bg.wasm` tem 21 MB, e o que está no ar tem 29 MB.
Em 4G ruim isso é meio minuto de tela branca *antes* da surpresa.

A mitigação é obrigatória e mora **fora do Rust**: uma capa em HTML puro dentro
de `docs/index.html`, ~2 KB, que aparece instantaneamente, lê só o nome de dentro
do fragmento em JavaScript e escreve

```
ANA,
TEM UMA COISA AQUI PRA VOCE
```

com um coração pulsando, enquanto os 21 MB chegam atrás. Espera vira
expectativa. Sem isso, o convidado vê uma tela branca e acha que o link está
quebrado — e o efeito é pior que nenhum link.

## 4. A mecânica: `O CAMINHO`

Céu escuro, poucos pontos de luz espalhados. Um deles pulsa.

1. O convidado arrasta o dedo até a estrela que pulsa. Ela acende.
2. Aparece o momento que o autor escreveu para ela — `14/03/2022 · o dia em que
   te vi de vermelho na fila do cinema`. O texto respira alguns segundos e sai.
3. Outra estrela começa a pulsar. Ao ligar a segunda na primeira, **a linha fica
   desenhada no céu.**
4. Repete pelas N estrelas.
5. Na última ligação a figura **fecha**. A linha inteira acende de uma vez, as
   estrelas viram luz, e o céu clareia.
6. A pergunta.

O que o convidado não sabe: as estrelas não estão espalhadas ao acaso. São os
vértices da figura que o autor escolheu. **A revelação é estrutural** — a
surpresa não é um texto dizendo "quer casar comigo", é a figura que a própria
pessoa acabou de traçar com o dedo sem saber o que era. É exatamente isso que um
slideshow de fotos não consegue fazer, e é a razão de existir da mecânica.

### 4.1 Decisões de desenho, e o porquê

- **A ordem é guiada, não livre.** A próxima estrela pulsa. Liberdade aqui só
  produz confusão e a chance de fechar a figura cedo, o que queimaria a
  revelação. O guia é o que garante que o final é o final.
- **Arrasto errado não faz nada.** Não erra: simplesmente não conecta. Sem
  vermelho, sem chacoalhada, sem som de negação. O repo tem uma camada de
  feedback boa justamente para punir (`interaction_animation.rs`); aqui ela é
  usada só para os acertos.
- **Sem tempo em nenhum lugar.** A pessoa pode parar no meio, chorar, ligar para
  a mãe e voltar. Progresso é salvo (§7.4).
- **Nada emocional depende de som.** No iPhone o switch lateral silencia a
  página e *nada* dentro dela reporta isso — o `AudioContext` continua lendo
  `running`. Está documentado no `CLAUDE.md` porque já custou uma depuração.
  Música é bônus; se não tocar, o pedido continua inteiro.
- **A figura fechada fica na tela no fim.** É a coisa que se printa e se manda
  pra família — o compartilhamento natural. O caminho de `navigator.share` já
  existe no repo (`interact_with_share_button`).
- **O gesto final é dos dois.** Depois da pergunta, um pad no rodapé:
  **dois polegares na tela ao mesmo tempo, três segundos**, e só então o `SIM`
  aparece. A resposta não é de uma pessoa só. Plano B, se o multi-touch não
  entregar dois ids distintos no build wasm (a medir): segurar com um dedo pelos
  mesmos três segundos. O gesto sobrevive à degradação; o desenho não depende
  dela.

### 4.2 Duração

N momentos = N estrelas. 3 a 8, escolha do autor. Cinco estrelas dão uns dois
minutos — o suficiente para virar um momento, pouco o bastante para ninguém
perder o fio.

## 5. O editor: `CRIAR O MEU`

O trabalho de "servir para outras pessoas" está aqui, e o risco é pedir demais.
Ninguém preenche trinta campos. São seis, nesta ordem, um por tela:

1. **Nome de quem vai receber.** É o que aparece na capa instantânea (§3.1).
2. **Os momentos.** De 3 a 8. Cada um é uma linha, com data opcional. Botão `+`.
   Contador de caracteres visível, porque o texto tem que caber na tela do
   celular sem quebrar (o repo já tem `theme::wrapped_text` para isso).
3. **A figura.** Coração, anel, casa, infinito, ou as duas iniciais. Vetores
   nossos, desenhados com lyon — nenhuma imagem, nenhum asset novo. O número de
   vértices da figura é o que limita o número de estrelas, então a escolha da
   figura e a quantidade de momentos conversam (§6.2).
4. **A pergunta.** Texto livre. Padrão `QUER CASAR COMIGO?`, com exemplos
   prontos a um toque: namoro, casamento, morar junto, pedido de desculpas.
   Texto livre é o que faz o app servir para o pedido de outra pessoa em vez de
   para o meu.
5. **A paleta.** Três ou quatro temas: noite, amanhecer, mar, jardim.
6. **O botão de resposta.** Só `SIM` (padrão, e o elegante), ou `SIM` mais um
   segundo botão brincalhão que foge do dedo — **opt-in**, porque tem gente que
   acha isso a melhor parte e gente que acha péssimo. O padrão é o sóbrio.

E no fim: `ENSAIAR` (joga do começo, sem gastar nada) e `GERAR O LINK`
(copiar / compartilhar). O ensaio não é luxo: é o que faz o autor descobrir que
o texto do momento 4 ficou comprido antes de entregar o celular.

## 6. Dados

### 6.1 O modelo

```rust
/// Tudo que o autor escreveu. É isto que viaja no fragmento da URL.
struct Proposal {
    guest: String,          // "Ana" — usado também pela capa em HTML
    moments: Vec<Moment>,   // 3..=8
    figure: Figure,         // Heart | Ring | House | Infinity | Initials(char, char)
    question: String,       // "QUER CASAR COMIGO?"
    palette: PaletteId,     // Night | Dawn | Sea | Garden
    playful_no: bool,       // o segundo botão que foge
}

struct Moment {
    date: Option<String>,   // livre, não validada: "14/03/2022", "verão de 2019"
    text: String,
}
```

### 6.2 A figura e as estrelas

Cada `Figure` é uma lista de vértices em coordenadas normalizadas, fechada. O
número de estrelas é o número de momentos, então a figura precisa entregar
vértices para qualquer N de 3 a 8: ela é definida como um **caminho amostrado** —
pega-se N pontos igualmente espaçados ao longo do contorno. Um coração com 3
estrelas ainda lê como coração; com 8, lê melhor. `Initials` é o único caso que
não amostra: as duas letras têm contagem própria de vértices, então ela restringe
N, e o editor avisa em vez de deixar o autor escolher algo que não fecha.

Isso é geometria pura e testável sem Bevy nenhum, do mesmo jeito que
`board.rs`, `mosaic_pattern.rs` e `wfc.rs` já são. Invariantes que merecem teste:
todo N de 3 a 8 devolve N pontos distintos; nenhum ponto cai fora da tela útil;
dois pontos consecutivos nunca ficam mais perto que o alvo de toque de 48 px; e
a figura fecha (o último liga no primeiro).

### 6.3 Codificação

`Proposal` → string `chave=valor` separada por `;` (o formato que o repo já usa
em `SavedRun` e no `share_request`) → base64url → fragmento.

Conta de tamanho: 6 momentos × ~60 caracteres, mais nome e pergunta, dá ~500
bytes; em base64url, ~700. Cabe folgado em qualquer mensageiro. Sem compressão na
v1 — ela só passaria a valer se entrassem fotos, e fotos não entram:

**Sem fotos, de propósito.** Foto exige hospedagem, hospedagem exige servidor, e
servidor exige moderação de conteúdo que estranhos vão subir. É o limite que eu
desenharia mesmo se fosse de graça. O que o app faz bem é texto e geometria.

## 7. Onde mora, e o que muda no repo

Mesmo repositório. O que se ganha de graça: `theme.rs`, `feedback.rs`
(`RevealIn`, `PopAnim`, texto flutuante são exatamente as primitivas que a
constelação precisa), `layout.rs` (`RelayoutEvent`), `audio.rs`, `storage.rs`, o
pipeline WASM/Pages inteiro, o service worker e o PWA. Um repo novo pagaria tudo
isso de novo para não ganhar nada.

```
src/
  proposal/
    mod.rs            ProposalPlugin
    model.rs          Proposal, Moment, Figure — puro, com testes
    figure.rs         os contornos e a amostragem de N vértices — puro, com testes
    encode.rs         Proposal <-> base64url — puro, com testes (ida e volta)
    sky/              o jogo: estrelas, arrasto, linhas, o fechamento
    editor/           as seis telas do autor (convenção de quatro partes do repo)
    ending/           a pergunta, o pad de dois polegares, o SIM
```

Três dos seis módulos são puros e testáveis sem app rodando, o que segue a
divisão que o `CLAUDE.md` já defende.

### 7.1 A fonte — o maior item de asset

`assets/digital7mono.ttf` é display de sete segmentos e **não tem acento**. É por
isso que o repo inteiro hoje escreve `HISTORICO` e `SEQUENCIA`. Para um pedido
isso não serve: "Voc quer casar comigo?" parece app quebrado exatamente no
instante em que menos pode parecer.

**Precisa de uma fonte nova com Latin-1 completo e licença OFL**, e a regra de
ASCII-sem-acento do `CLAUDE.md` passa a valer só para o Color Puzzle. Duas fontes
na árvore, cada uma com o seu escopo — e o `CLAUDE.md` precisa dizer isso, porque
hoje ele afirma a regra sem ressalva.

### 7.2 Roteamento na entrada

`main.rs` ganha uma leitura do fragmento antes de escolher o estado inicial. Hoje
`web-sys` está ligado só com `["Window", "Storage"]`; ler `location.hash` pede a
feature `Location` — ou, melhor, vai pela ponte que já existe: `index.html` já
conversa com o Rust por `localStorage` (`color_puzzle.share_request`), e a capa
instantânea já vai ler o fragmento em JavaScript de qualquer forma. Ela escreve o
payload numa chave, o Rust lê por `storage::load`, e nenhuma feature nova entra.
Menos superfície, e o JS que precisa existir já existe.

### 7.3 Tema claro

`theme.rs` hoje é um tema só, violeta quase preto, e o `CLAUDE.md` proíbe paletas
por tela — com razão. As paletas do autor entram como **variantes do tema**, uma
`PaletteId` que `theme.rs` resolve, não como cores soltas no módulo do pedido.

### 7.4 Retomar

O convidado pode tocar em "voltar" sem querer. O progresso (quantas estrelas
ligadas) vai para `localStorage` com chave derivada de um hash do fragmento, para
retomar de onde parou em vez de recomeçar. Um pedido que recomeça do zero na
terceira estrela é um pedido perdido.

### 7.5 Armadilhas do repo que se aplicam direto

- **Nunca despachar um `Button` vivo dentro do `Update`** (`B0003`). O editor é a
  tela mais dinâmica que o repo vai ter — adiciona e remove campos de momento —
  então tudo que despacha vai para `PostUpdate`, como `spawn_pagination_itens` já
  faz.
- **`SystemTime::now()` explode no wasm.** Qualquer coisa de tempo usa
  `Res<Time>`.
- **Toda tela que se dimensiona pela janela trata `RelayoutEvent`** — no
  navegador o canvas muda de tamanho um ou dois frames depois de montada a tela.
- **`cargo check --target wasm32-unknown-unknown` antes de acreditar em qualquer
  coisa**, e `RUSTFLAGS` não pode ser setado porque sobrescreve o
  `getrandom_backend` do `.cargo/config.toml`.

## 8. Riscos, em ordem de tamanho

1. **A fonte** (§7.1). Item de asset mais importante do projeto. Sem acento não
   há pedido.
2. **Os 21 MB** no fluxo de link (§3.1). A capa instantânea em HTML é obrigatória,
   não enfeite.
3. **Multi-touch no wasm** para o gesto dos dois polegares. A medir no começo da
   implementação; o plano B (um dedo, mesmos 3 s) já está no desenho.
4. **Reload no meio** (§7.4).
5. **A figura com N baixo.** Um coração de 3 vértices pode não ler como coração.
   Mitigação: o editor mostra a prévia da figura enquanto o autor adiciona
   momentos, e sugere um mínimo por figura.

## 9. Fases

1. **Núcleo puro, sem Bevy.** `model.rs`, `figure.rs`, `encode.rs` e os testes.
   Dá para rodar com `cargo test` sem GPU, sem sistema gráfico, sem nada.
2. **O céu.** A tela do convidado com uma `Proposal` fixa no código: estrelas,
   arrasto, linhas, o fechamento, a pergunta. É aqui que se descobre se a
   mecânica emociona — e é aqui que a fonte nova entra.
3. **O final.** O pad de dois polegares (medindo o multi-touch no navegador
   primeiro), o `SIM`, a figura que fica na tela, o compartilhamento.
4. **O editor.** As seis telas, o ensaio, o link.
5. **A capa instantânea** em `index.html` e o roteamento por fragmento.
6. **PWA e offline.** Conferir que o service worker cobre a fonte nova e que uma
   URL com fragmento cai na navegação certa quando está sem sinal.

A fase 2 é o ponto de decisão: se o céu não emocionar com uma `Proposal` chumbada
no código, o editor não salva o projeto, e é melhor descobrir isso antes de
escrever seis telas de formulário.
